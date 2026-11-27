; A SELL NEVER DRAINS THE POOL AND NEVER LOWERS ITS PRODUCT, AND ITS FEE IS NEVER MORE THAN IT RECEIVES (law.rs traded, Side::Sell), over the
; unbounded integers. The pool trades ALL the claims: gross is the floor of quote * amount / (base + amount), and the pool becomes
; (base + amount, quote - gross). The fee is $WARM beside the pool, taken from what left it: fee, the ceiling of gross * k / 10000 for a fee
; of k bp under 10000 (law.rs fee_of); the purse gets gross - fee. Claim: gross < quote, (base + amount) * (quote - gross) >= base * quote, and
; 0 <= fee <= gross, so the purse never gets less than nothing.
(set-logic QF_NIA)
(declare-const base Int) (declare-const quote Int) (declare-const amount Int) (declare-const k Int) (declare-const fee Int) (declare-const gross Int)
(assert (and (> base 0) (> quote 0) (> amount 0) (>= k 0) (< k 10000) (>= gross 0)))
(assert (<= (* gross (+ base amount)) (* quote amount)))
(assert (< (* quote amount) (* (+ gross 1) (+ base amount))))
(assert (and (>= (* fee 10000) (* gross k)) (< (* (- fee 1) 10000) (* gross k))))
(assert (not (and (< gross quote) (>= (* (+ base amount) (- quote gross)) (* base quote)) (<= 0 fee) (<= fee gross))))
(check-sat)
