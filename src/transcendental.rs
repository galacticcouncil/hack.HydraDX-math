#![allow(clippy::result_unit_err)]

use core::convert::From;
use core::ops::{AddAssign, BitOrAssign, ShlAssign, Shr, ShrAssign};
use fixed::traits::FixedUnsigned;
use fixed::traits::ToFixed;

/// right-shift with rounding
fn rs<T>(operand: T) -> T
where
    T: FixedUnsigned,
{
    let lsb = T::from_num(1) >> T::FRAC_NBITS;
    (operand >> 1) + (operand & lsb)
}

/// base 2 logarithm assuming self >=1
fn log2_inner<S, D>(operand: S) -> D
where
    S: FixedUnsigned + PartialOrd<D>,
    D: FixedUnsigned,
    D::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
    let two = D::from_num(2);
    let mut x = operand;
    let mut result = D::from_num(0).to_bits();
    let lsb = (D::from_num(1) >> D::FRAC_NBITS).to_bits();

    while x >= two {
        result += lsb;
        x = rs(x);
    }

    if x == D::from_num(1) {
        return D::from_num(result);
    };

    for _i in (0..D::FRAC_NBITS).rev() {
        x *= x;
        result <<= lsb;
        if x >= two {
            result |= lsb;
            x = rs(x);
        }
    }
    D::from_bits(result)
}

/// base 2 logarithm
///
/// Returns tuple(D,bool) where bool indicates whether D is negative. This happens when operand is < 1.
pub fn log2<S, D>(operand: S) -> Result<(D, bool), ()>
where
    S: FixedUnsigned,
    D: FixedUnsigned + From<S>,
    D::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
    if operand <= S::from_num(0) {
        return Err(());
    };

    let operand = D::from(operand);
    if operand < D::from_num(1) {
        let inverse = D::from_num(1).checked_div(operand).unwrap();
        return Ok((log2_inner::<D, D>(inverse), true));
    };
    Ok((log2_inner::<D, D>(operand), false))
}

/// natural logarithm
/// Returns tuple(D,bool) where bool indicates whether D is negative. This happens when operand is < 1.
pub fn ln<S, D>(operand: S) -> Result<(D, bool), ()>
where
    S: FixedUnsigned,
    D: FixedUnsigned + From<S>,
    D::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    S::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShrAssign + Shr,
{
    let log2_e = S::from_str("1.442695").map_err(|_| ())?;
    let log_result = log2::<S, D>(operand)?;
    Ok((log_result.0 / D::from(log2_e), log_result.1))
}

/// exponential function e^(operand)
/// neg - bool indicates that operand is negative value.
pub fn exp<S, D>(operand: S, neg: bool) -> Result<D, ()>
where
    S: FixedUnsigned + PartialOrd<D>,
    D: FixedUnsigned + PartialOrd<S> + From<S>,
{
    if operand.is_zero() {
        return Ok(D::from_num(1));
    };
    if operand == S::from_num(1) {
        //TODO: make this as const somewhere
        let e = S::from_str("2.718281828459045235360287471352662497757").map_err(|_| ())?;
        return Ok(D::from(e));
    };

    let operand = D::from(operand);
    let mut result = operand + D::from_num(1);
    let mut term = operand;

    result = (2..D::FRAC_NBITS).try_fold(result, |acc, i| -> Result<D, ()> {
        term = term.checked_mul(operand).ok_or(())?;
        term = term.checked_div(D::from_num(i)).ok_or(())?;
        acc.checked_add(term).ok_or(())
    })?;

    if neg {
        result = D::from_num(1).checked_div(result).ok_or(())?;
    }

    Ok(result)
}

pub fn pow<S, D>(operand: S, exponent: S) -> Result<D, ()>
where
    S: FixedUnsigned + PartialOrd<D>,
    D: FixedUnsigned + From<S>,
    D::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    S::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign + Shr + ShrAssign,
{
    if operand.is_zero() {
        return Ok(D::from_num(0));
    };
    if exponent == S::from_num(0) {
        return Ok(D::from_num(1));
    };
    if exponent == S::from_num(1) {
        return Ok(D::from(operand));
    };

    let (r, neg) = ln::<S, D>(operand)?;

    let r: D = r.checked_mul(exponent.into()).ok_or(())?;
    let r: D = exp(r, neg)?;

    let (result, oflw) = r.overflowing_to_num::<D>();
    if oflw {
        return Err(());
    };
    Ok(result)
}

/// power with integer exponent
pub fn powi<S, D>(operand: S, exponent: u32) -> Result<D, ()>
where
    S: FixedUnsigned,
    D: FixedUnsigned + From<S>,
{
    if operand == S::from_num(0) {
        return Ok(D::from_num(0));
    };
    if exponent == 0 {
        return Ok(D::from_num(1));
    };
    if exponent == 1 {
        return Ok(D::from(operand));
    };
    let operand = D::from(operand);

    let r = (1..exponent).try_fold(operand, |acc, _| acc.checked_mul(operand));

    r.ok_or(())
}

#[cfg(test)]
mod tests {
    use crate::types::FixedBalance;
    use core::str::FromStr;
    use fixed::traits::LossyInto;
    use fixed::types::U64F64;

    use super::{exp, log2, pow, powi};

    #[test]
    fn exp_works() {
        type S = U64F64;
        type D = U64F64;

        let e = S::from_str("2.718281828459045235360287471352662497757").unwrap();

        let zero = S::from_num(0);
        let one = S::from_num(1);
        let two = S::from_num(2);

        assert_eq!(exp::<S, D>(zero, false), Ok(D::from_num(one)));
        assert_eq!(exp::<S, D>(one, false), Ok(D::from_num(e)));
        assert_eq!(
            exp::<S, D>(two, false),
            Ok(D::from_str("7.3890560989306502265").unwrap())
        );
        assert_eq!(
            exp::<S, D>(two, true),
            Ok(D::from_str("0.13533528323661269186").unwrap())
        );
    }

    #[test]
    fn log2_works() {
        type S = U64F64;
        type D = U64F64;

        let zero = S::from_num(0);
        let one = S::from_num(1);
        let two = S::from_num(2);
        let four = S::from_num(4);

        assert_eq!(log2::<S, D>(zero), Err(()));

        assert_eq!(log2(two), Ok((D::from_num(one), false)));
        assert_eq!(log2(one / four), Ok((D::from_num(two), true)));
        assert_eq!(log2(S::from_num(0.5)), Ok((D::from_num(one), true)));
        assert_eq!(log2(S::from_num(1.0 / 0.5)), Ok((D::from_num(one), false)));
    }

    #[test]
    fn powi_works() {
        type S = U64F64;
        type D = U64F64;

        let zero = S::from_num(0);
        let one = S::from_num(1);
        let two = S::from_num(2);
        let four = S::from_num(4);

        assert_eq!(powi(two, 0), Ok(D::from_num(one)));
        assert_eq!(powi(zero, 2), Ok(D::from_num(zero)));
        assert_eq!(powi(two, 1), Ok(D::from_num(2)));
        assert_eq!(powi(two, 3), Ok(D::from_num(8)));
        assert_eq!(powi(one / four, 2), Ok(D::from_num(0.0625)));
        assert_eq!(powi(S::from_num(2), 2), Ok(D::from_num(4)));
    }

    #[test]
    fn pow_works() {
        type S = FixedBalance;
        type D = FixedBalance;
        let zero = S::from_num(0);
        let one = S::from_num(1);
        let two = S::from_num(2);
        let three = S::from_num(3);
        let four = S::from_num(4);

        assert_eq!(pow::<S, D>(two, zero), Ok(one.into()));
        assert_eq!(pow::<S, D>(zero, two), Ok(zero.into()));

        let result: f64 = pow::<S, D>(two, three).unwrap().lossy_into();
        assert_relative_eq!(result, 8.0, epsilon = 1.0e-6);

        let result: f64 = pow::<S, D>(one / four, two).unwrap().lossy_into();
        assert_relative_eq!(result, 0.0625, epsilon = 1.0e-6);

        assert_eq!(pow::<S, D>(two, one), Ok(two.into()));

        let result: f64 = pow::<S, D>(one / four, one / two).unwrap().lossy_into();
        assert_relative_eq!(result, 0.5, epsilon = 1.0e-6);

        assert_eq!(
            pow(S::from_num(22.1234), S::from_num(2.1)),
            Ok(D::from_num(667.097035126091))
        );

        assert_eq!(
            pow(S::from_num(0.986069911074), S::from_num(1.541748732743)),
            Ok(D::from_num(0.978604513883))
        );
    }
}
