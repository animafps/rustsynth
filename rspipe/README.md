# vspipe-rs

```txt
VapourSynth script processor using rustsynth

Usage: rspipe [OPTIONS] [script] [outfile]

Arguments:
  [script]   VapourSynth script file (.vpy)
  [outfile]  Output file (use '-' for stdout, '--' for no output)

Options:
  -a, --arg <key=value>     Argument to pass to the script environment
  -s, --start <N>           Set output frame/sample range start
  -e, --end <N>             Set output frame/sample range end (inclusive)
  -o, --outputindex <N>     Select output index [default: 0]
  -r, --requests <N>        Set number of concurrent frame requests
  -c, --container <FORMAT>  Add headers for the specified format to the output [possible values: y4m, wav, w64]
  -p, --progress            Print progress to stderr
  -i, --info                Print all set output node info and exit
  -v, --version             Show version info and exit
  -h, --help                Print help
```

## Examples

Show script info:

`rspipe --info script.vpy`

Write to stdout:

`rspipe [options] script.vpy -`

Request all frames but don’t output them:

`rspipe [options] script.vpy --`

Write frames 5-100 to file:

`rspipe --start 5 --end 100 script.vpy output.raw`

Pipe to x264:

`rspipe script.vpy - -c y4m | x264 --demuxer y4m -o script.mkv -`

Pass values to a script:

`rspipe --arg deinterlace=yes --arg "message=fluffy kittens" script.vpy output.raw`
