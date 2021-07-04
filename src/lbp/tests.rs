use crate::lbp::lbp;

use crate::MathError::{Overflow, ZeroDuration, ZeroInReserve, ZeroOutWeight};

#[test]
fn spot_price_should_work() {
    let cases = vec![
        (1000, 2000, 500, 500, 100, Ok(200), "Easy case"),
        (0, 0, 0, 0, 100, Err(ZeroInReserve), "Zero reserves and weights"),
        (0, 1, 1, 1, 1, Err(ZeroInReserve), "Zero sell_reserve"),
        (1, 0, 1, 1, 1, Ok(0), "Zero buy_reserve"),
        (1, 1, 0, 1, 1, Ok(0), "Zero amount"),
        (u128::MAX, u128::MAX - 1, 1, 1, 1, Ok(0), "Truncated result"),
        (
            1,
            u128::MAX,
            u128::MAX,
            u128::MAX,
            u128::MAX,
            Err(Overflow),
            "Overflow weights",
        ),
    ];

    for case in cases {
        assert_eq!(
            lbp::calculate_spot_price(case.0, case.1, case.2, case.3, case.4),
            case.5,
            "{}",
            case.6
        );
    }
}

#[test]
fn out_given_in_should_work() {
    let cases = vec![
        (1000, 2000, 500, 500, 100, Ok(182), "Easy case"),
        (0, 0, 0, 0, 100, Err(ZeroOutWeight), "Zero reserves and weights"),
        (
            u128::MAX,
            u128::MAX,
            u128::MAX,
            u128::MAX,
            u128::MAX,
            Ok(170141183460469231731687303715884105728),
            "max",
        ),
        //(1, 0, 1, 1, 0, Err(Overflow), "Zero out reserve and amount"), TOODO: check this why it does not overflow anymore
        (0, 0, 1, 1, u128::MAX, Ok(0), "Zero buy reserve and sell reserve"),
    ];

    for case in cases {
        assert_eq!(
            lbp::calculate_out_given_in(case.0, case.1, case.2, case.3, case.4),
            case.5,
            "{}",
            case.6
        );
    }
}

#[test]
fn in_given_out_should_work() {
    let prec: u128 = 1_000_000_000_000u128;
    let cases = vec![
        (1000, 2000, 500, 500, 100, Ok(53), "Easy case"),
        (
            100 * prec,
            20 * prec,
            50 * prec,
            100 * prec,
            prec,
            Ok(10803324099600),
            "Easy case",
        ),
        (
            100 * prec,
            20 * prec,
            100 * prec,
            50 * prec,
            prec,
            Ok(2597864120100),
            "Easy case",
        ),
        (
            100 * prec,
            340 * prec,
            100 * prec,
            1200 * prec,
            2 * prec,
            Ok(7336295198400),
            "Easy case",
        ),
        (0, 0, 0, 0, 100, Err(Overflow), "Zero reserves and weights"),
    ];

    for case in cases {
        assert_eq!(
            lbp::calculate_in_given_out(case.0, case.1, case.2, case.3, case.4),
            case.5,
            "{}",
            case.6
        );
    }
}

#[test]
fn linear_weights_should_work() {
    let u32_cases = vec![
        (100u32, 200u32, 1_000u128, 2_000u128, 170u32, Ok(1_700), "Easy case"),
        (
            100u32,
            200u32,
            2_000u128,
            1_000u128,
            170u32,
            Ok(1_300),
            "Easy decreasing case",
        ),
        (
            100u32,
            200u32,
            2_000u128,
            2_000u128,
            170u32,
            Ok(2_000),
            "Easy constant case",
        ),
        (
            100u32,
            200u32,
            1_000u128,
            2_000u128,
            100u32,
            Ok(1_000),
            "Initial weight",
        ),
        (
            100u32,
            200u32,
            2_000u128,
            1_000u128,
            100u32,
            Ok(2_000),
            "Initial decreasing weight",
        ),
        (
            100u32,
            200u32,
            2_000u128,
            2_000u128,
            100u32,
            Ok(2_000),
            "Initial constant weight",
        ),
        (100u32, 200u32, 1_000u128, 2_000u128, 200u32, Ok(2_000), "Final weight"),
        (
            100u32,
            200u32,
            2_000u128,
            1_000u128,
            200u32,
            Ok(1_000),
            "Final decreasing weight",
        ),
        (
            100u32,
            200u32,
            2_000u128,
            2_000u128,
            200u32,
            Ok(2_000),
            "Final constant weight",
        ),
        (
            200u32,
            100u32,
            1_000u128,
            2_000u128,
            170u32,
            Err(Overflow),
            "Invalid interval",
        ),
        (
            100u32,
            100u32,
            1_000u128,
            2_000u128,
            100u32,
            Err(ZeroDuration),
            "Invalid interval",
        ),
        (
            100u32,
            200u32,
            1_000u128,
            2_000u128,
            10u32,
            Err(Overflow),
            "Out of bound",
        ),
        (
            100u32,
            200u32,
            1_000u128,
            2_000u128,
            210u32,
            Err(Overflow),
            "Out of bound",
        ),
    ];
    let u64_cases = vec![
        (100u64, 200u64, 1_000u128, 2_000u128, 170u64, Ok(1_700), "Easy case"),
        (
            100u64,
            u64::MAX,
            1_000u128,
            2_000u128,
            200u64,
            Err(Overflow),
            "Interval too long",
        ),
    ];

    for case in u32_cases {
        assert_eq!(
            lbp::calculate_linear_weights(case.0, case.1, case.2, case.3, case.4),
            case.5,
            "{}",
            case.6
        );
    }
    for case in u64_cases {
        assert_eq!(
            lbp::calculate_linear_weights(case.0, case.1, case.2, case.3, case.4),
            case.5,
            "{}",
            case.6
        );
    }
}