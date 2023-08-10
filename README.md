# crn

[![Crates.io](https://img.shields.io/crates/v/crn.svg)](https://crates.io/crates/crn)
[![Documentation](https://docs.rs/crn/badge.svg)](https://docs.rs/crn)

`crn` can simulate both stochastic and deterministic CRNs with `StoCrn` and `DetCrn`, respectively.

To create your own CRN, first declare the initial counts of each molecule, and then add the reactions, each optionally followed by a rate parameter (10 in the first reaction):

```rust
let crn_string = "
a = 10;
b = 5;
c = 0;
a + b -> 2c : 10;
c -> 3b;
";
```

Pass the whole string to the parser:

`let mut crn = StoCrn::parse(crn_string).unwrap()`

Now let's simulate it for 3 seconds of virtual time:

`let data = crn.simulate_history(3.0);`

Some premade CRNs can be found in the `presets` module.

Run this for a graphical demonstration!

`cargo run --release --example gui`

Note: deterministic simulations tend to be unstable with large numbers -- I'm still working on this. Try scaling down all initial amounts (can be noninteger, unlike stochastic simulations) if you're having issues.

![gui demo](media/1691519892.png)
