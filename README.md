# avirus

`avirus` is a library for manipulating AVI files for purposes such as glitch art.

```toml,ignore
[dependencies]
avirus = "0.2.4"
```

## Examples

`avirus::AVI` takes an existing AVI file and loads it into memory for manipulation. `avirus::frames` exposes a `meta` field, which holds simple structures with metadata about a frame. This field can be iterated over, and modified to create odd effects in the output file. When the AVI file's `output()` function is called, a new file will be rebuilt with the modified metadata.

```rust
extern crate avirus;

use avirus::AVI;
use avirus::frame::Frame;

fn main() {
    let mut avi = AVI::new("sample.avi").unwrap();
    let mut new_meta: Vec<Frame> = Vec::new();
    for frame in &mut avi.frames.meta {
        if frame.is_pframe() || frame.is_audioframe() {
            for _ in 0..3 {
                new_meta.push(*frame);
            }
        }
        else {
            new_meta.push(*frame);
        }
    }
    avi.frames.meta = new_meta;
    avi.output("sample_output.avi").unwrap();
}
```
