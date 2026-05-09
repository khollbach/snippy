use anyhow::Result;

mod compress;
mod decompress;
mod varint;

/*
TODO: re-shape compress API as follows:

fn compress(input: impl Read, output: impl Write) -> io::Result<()>
fn max_compressed_len(input_len: u32) -> u64

some details tbd:
- maybe m.c.l. has a wrapper fxn that works with usize instead;
  or maybe just example code; or maybe a compress_to_vec wrapper? idk
- in the compress-to-vec example/wrapper, you probably either use a cursor
  to know how much was written, or maybe you add that to the return value of
  `compress`
- could also have an API that lets you re-use buffers/tables/etc (details = ?),
  in case you're compressing many separate streams

Q:
- it would be good to be sure that this API elides extra copies of the input buffer
  in the case that the input is &[u8] (fwiw, I'm *guessing* it would (?))
- is this API nice enough for everyday use?
- what do other compression libs do?
*/

pub fn compress(input: &[u8]) -> Vec<u8> {
    compress::compress(input)
}

/*
TODO: re-shape decompress API:

fn decompress(input: impl Read) -> Result<Vec<u8>>
    - can fail to allocate, in which case, panics

    or, to allow buffer re-use:

struct Decompressor<R>
    fn new(input: impl Read) -> Self
    fn uncompressed_len(&mut self) -> Result<u32>
    fn decompress(self, buf: &mut [u8]) -> Result<usize>  (must-use?)
    - fails immediately if buffer too small

Qs:
- should we also include an output:Write API?
  - if we did, we'd want to make sure that copies get elided in the special case
    of output-is-a-vec (since the internal buffer is identical in that case)
- related: API usability -- what do (most?) (different?) people _want_ the API to feel like?
  - see existing compression libs for ideas!
- (unimportant, but maybe interesting): could we design an API that worked for large files
  on 16-bit systems? How would it compare to the above?

---

I guess one option is to do something like:

fn decompress(input: impl Read, output: impl Write + Seek + Read) -> Result<()>  (return bytes written?)

Seems like the above could work. I'm guessing you could use a cursor around a
vec to get this to work like the simpler APIs.

The thing that would be cool about this is using it with a (larger-than-RAM)
file as the output buffer. But I think to get it to be performant you'd have
to think deliberately about buffering -- you can't just combine BufWriter and
BufReader both wrapping the file, so you'd probably want your own thing? Details
unclear, but could be interesting to think about.
*/

pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    decompress::decompress(compressed)
}
