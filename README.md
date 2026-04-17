# Omniglot MPK Runtime

## How to run our experimental tests

Most of these tests should exit with a successful, zero error code.

However, some of them may exit with a non-zero code due to the nature of the tests (e.g., intentionally causing a segmentation fault).

Test 3 is expected to exit with a non-zero code due to the intentional segmentation fault, while the others should exit successfully.

```bash
./experiment.sh
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
