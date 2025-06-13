# resizer

So basically I got tired of convert and imagemagik's inablity to resize large 100MB+ images.

# Usage

### Single file
`resizer input.jpg --resize "50%" `

### Multiple files
`resizer *.jpg --resize "800x600" -o output/ `

### Directory processing
`resizer ./assets --resize "20%" `

### Force overwrite
`resizer *.png --resize "300x300" -f `
