// THE SPACE ROUTER'S LAW OF $WARM, on chain: what a dollar spent at an instant mines. A port, with no float, of the router's own law
// (`embertide-app/ui/intro/dyson-society/app/space-router-law.mjs`: WARM_PER_USD, HALVING_MONTHS, halvingAt, eraAt), held to it row by
// row by a table its own functions made (`tests/law-parity.json`, read by `tests/law.rs`). His words: Oct 27 2026 "1$ = 1$WARM? to begin
// with"; Oct 28 13:50 "we should give $WARM only when money is spent in the SPACE-ROUTER"; Oct 29 "Halve by date", "Every 6 months";
// Nov 21 17:34:37 "it is supposed to be uncapped supply, prompt-2-mine; the issuance needs to be live and continuous based on the
// space-router utilization".
//
// Pure: no account, no clock, no allocation, no dependency. THE TWO PROGRAMS IN ONE WORLD (poleis_treasury/tests/council.rs) `include!`s
// this file whole, so the world's test computes what a spend mines by the very text the program runs, never a copy. Outer comments only:
// an included file can hold no inner attribute.

/// $WARM a dollar mines at the opening, and before it.
pub const WARM_PER_USD: u64 = 1;
/// The calendar months (UTC) from one halving to the next, counted from the opening.
pub const HALVING_MONTHS: u64 = 6;
/// USDC counts micros (6 decimals) and $WARM nanos (9): a micro-dollar at the opening's rate mines a thousand nanos of $WARM.
pub const NANOS_PER_MICRO: u64 = 1_000;
/// A day, in the law's milliseconds.
pub const DAY_MS: u64 = 86_400_000;

/// The year, month (1 to 12) and day (1 to 31) of a day counted from 1970-01-01, in the proleptic Gregorian calendar, as JavaScript's
/// `Date` reads it in UTC (H. Hinnant's `civil_from_days`, in unsigned integers: every instant here is at or after the epoch).
pub fn civil_of(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// The day counted from 1970-01-01 of a date at or after it (`days_from_civil`); None past what a u64 counts.
pub fn days_of(year: u64, month: u64, day: u64) -> Option<u64> {
    let y = if month <= 2 { year.checked_sub(1)? } else { year };
    let era = y / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era.checked_mul(146_097)?.checked_add(doe)?.checked_sub(719_468)
}

/// The days of a month (1 to 12) of a year.
pub fn month_days(year: u64, month: u64) -> u64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The n-th halving (`halvingAt`): the opening plus n × HALVING_MONTHS calendar months, UTC, at the opening's time of day; a day past its
/// month's end lands on that month's last day (Aug 31 + 6 months is Feb 28, or 29 in a leap year). Always counted from the opening, never
/// from the halving before. None past what a u64 counts in milliseconds (an instant no chain reaches).
pub fn halving_at(opened_ms: u64, n: u64) -> Option<u64> {
    let (days, of_day) = (opened_ms / DAY_MS, opened_ms % DAY_MS);
    let (y, m, d) = civil_of(days);
    let months = (m - 1).checked_add(n.checked_mul(HALVING_MONTHS)?)?;
    let (year, month) = (y.checked_add(months / 12)?, months % 12 + 1);
    let last = month_days(year, month);
    days_of(year, month, if d < last { d } else { last })?.checked_mul(DAY_MS)?.checked_add(of_day)
}

/// The era of an instant (`eraAt`): 0 before the first halving, and before the opening; n from the n-th halving to the next. The same
/// steps as the router's: an estimate by calendar months, then walked down and up to the halvings that bound the instant.
pub fn era_at(at_ms: u64, opened_ms: u64) -> u64 {
    if at_ms < opened_ms {
        return 0;
    }
    let (ya, ma, _) = civil_of(at_ms / DAY_MS);
    let (yo, mo, _) = civil_of(opened_ms / DAY_MS);
    let mut n = ((ya * 12 + ma) - (yo * 12 + mo)) / HALVING_MONTHS;
    while n > 0 && matches!(halving_at(opened_ms, n), Some(h) if h > at_ms) {
        n -= 1;
    }
    while matches!(halving_at(opened_ms, n + 1), Some(h) if h <= at_ms) {
        n += 1;
    }
    n
}

/// What a spend of `usd_micros` at `at_ms` mines, in nanos of $WARM: `usd_micros × 1,000 × WARM_PER_USD >> era`, uncapped, forever. The
/// shift floors: a spend mines the law's exact amount when it is a whole nano, and never the half of a nano the router's float would round
/// up (`warmMined` rounds to the nearest nano; from the fourth halving the two can differ by that one nano, and the chain's is the lower).
/// A u128, so no spend's product can wrap; the caller holds it to a u64.
pub fn warm_nanos(usd_micros: u64, at_ms: u64, opened_ms: u64) -> u128 {
    let whole = usd_micros as u128 * (NANOS_PER_MICRO * WARM_PER_USD) as u128;
    let era = era_at(at_ms, opened_ms);
    if era >= 128 {
        0
    } else {
        whole >> era
    }
}
