# Changelog

## 0.2.0 (2026-10-29)

- **Breaking:** depends on primase-foundations 0.3.0; `le::LE` is its `bytes::Endian`, so that type changes with it.
- `Install::binary`, `stored` and `resource` take an open file of the archive from a pool the install keeps (and open
  one only when none is spare) instead of opening and closing the archive file for every read: threads reading one
  archive file each have a file to themselves. On a GTA V Enhanced install reading resources opened a file every time
  (about 65 us an open and close here, and 32 % of the CPU of a region read).
- `Meta::root`, `root_scalars` and the other walks note the structures they reach in a bit per byte of each block, not
  in a `BTreeSet`; the same errors, the same trees.
- `examples/read_bench.rs`: a timed read of an install's files on N threads with a digest of everything read, for
  comparing changes to the archive read path.

## 0.1.0 (2026-10-27)

- First release: GTA V's RPF7 archives, resources, META, drawables, textures, collision bounds, clips and shader containers. The archive keys are the caller's to supply.
