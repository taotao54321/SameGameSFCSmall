#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! chmin {
    ($xmin:expr, $x:expr) => {{
        if $x < $xmin {
            $xmin = $x;
            true
        } else {
            false
        }
    }};
}
pub(crate) use chmin;

macro_rules! chmax {
    ($xmax:expr, $x:expr) => {{
        if $x > $xmax {
            $xmax = $x;
            true
        } else {
            false
        }
    }};
}
pub(crate) use chmax;
