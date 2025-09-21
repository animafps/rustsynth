use rustsynth::format::ColorFamily;
use rustsynth::{format::VideoInfo, frame::Frame};
use std::fs::File;
use std::io::{self, BufWriter, Write};

pub struct OutputWriter {
    writer: Box<dyn Write>,
    container_format: Option<String>,
    wrote_header: bool,
}

impl OutputWriter {
    pub fn new(outfile: &str, container: Option<&String>) -> io::Result<Self> {
        let writer: Box<dyn Write> = match outfile {
            "-" => Box::new(BufWriter::with_capacity(1024 * 1024, io::stdout())),
            "--" => Box::new(io::sink()),
            path => Box::new(BufWriter::with_capacity(1024 * 1024, File::create(path)?)),
        };

        Ok(OutputWriter {
            writer,
            container_format: container.cloned(),
            wrote_header: false,
        })
    }

    pub fn write_header(&mut self, video_info: &VideoInfo) -> io::Result<()> {
        if let Some(container) = &self.container_format {
            match container.as_str() {
                "y4m" => self.write_y4m_header(video_info)?,
                "wav" | "w64" => {
                    // Audio container headers would go here
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Audio containers not yet implemented",
                    ));
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported container format: {}", container),
                    ));
                }
            }
        }
        self.wrote_header = true;
        Ok(())
    }

    fn write_y4m_header(&mut self, video_info: &VideoInfo) -> io::Result<()> {
        // Y4M header format: YUV4MPEG2 W<width> H<height> F<fps_num>:<fps_den> Ip A0:0 C420jpeg XYSCSS=420JPEG
        let format_tag = match (
            video_info.format.color_family,
            video_info.format.bits_per_sample,
            video_info.format.sub_sampling_w,
            video_info.format.sub_sampling_h,
        ) {
            (ColorFamily::YUV, 8, 1, 1) => "C420jpeg",
            (ColorFamily::YUV, 8, 1, 0) => "C422",
            (ColorFamily::YUV, 8, 0, 0) => "C444",
            (ColorFamily::YUV, 10, 1, 1) => "C420p10",
            (ColorFamily::YUV, 10, 1, 0) => "C422p10",
            (ColorFamily::YUV, 10, 0, 0) => "C444p10",
            (ColorFamily::YUV, 12, 1, 1) => "C420p12",
            (ColorFamily::YUV, 12, 1, 0) => "C422p12",
            (ColorFamily::YUV, 12, 0, 0) => "C444p12",
            (ColorFamily::YUV, 16, 1, 1) => "C420p16",
            (ColorFamily::YUV, 16, 1, 0) => "C422p16",
            (ColorFamily::YUV, 16, 0, 0) => "C444p16",
            _ => "C420jpeg", // default fallback
        };

        writeln!(
            self.writer,
            "YUV4MPEG2 W{} H{} F{}:{} Ip A0:0 {}",
            video_info.width, video_info.height, video_info.fps_num, video_info.fps_den, format_tag
        )?;

        Ok(())
    }

    pub fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        if let Some(container) = &self.container_format {
            match container.as_str() {
                "y4m" => self.write_y4m_frame(frame)?,
                "wav" | "w64" => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Audio containers not yet implemented",
                    ));
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid container",
                    ));
                }
            }
        } else {
            self.write_raw_frame(frame)?;
        }
        Ok(())
    }

    fn write_y4m_frame(&mut self, frame: &Frame) -> io::Result<()> {
        // Y4M frame header
        writeln!(self.writer, "FRAME")?;

        // Write raw frame data
        self.write_raw_frame(frame)?;
        Ok(())
    }

    fn write_raw_frame(&mut self, frame: &Frame) -> io::Result<()> {
        let format = frame.get_video_format().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Frame has no video format")
        })?;
        let num_planes = format.num_planes;

        // Write each plane
        for plane in 0..num_planes {
            let data_ptr = frame.get_read_ptr(plane);
            let stride = frame.get_stride(plane) as usize;
            let width = frame.get_width(plane) as usize;
            let height = frame.get_height(plane) as usize;
            let bytes_per_sample = format.bytes_per_sample as usize;

            // Create slice from pointer
            let data = unsafe { std::slice::from_raw_parts(data_ptr, stride * height) };

            // Write line by line to handle stride properly
            for y in 0..height {
                let line_start = y * stride;
                let line_end = line_start + width * bytes_per_sample;
                self.writer.write_all(&data[line_start..line_end])?;
            }
        }

        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
