; A BUY NEVER DRAINS THE POOL AND NEVER LOWERS ITS PRODUCT, AND ITS FEE IS NEVER MORE THAN IT PAYS (law.rs traded, Side::Buy), over the
; unbounded integers. The fee is $WARM beside the pool: fee, the ceiling of amount * k / 10000 for a fee of k bp under 10000 (law.rs fee_of),
; comes out of what the buy pays, and only net = amount - fee reaches the pool: out is the floor of base * net / (quote + net), and the pool
; becomes (base - out, quote + net). Claim: 0 <= fee <= amount, 0 <= net <= amount (the pool never receives more than the buy pays, nor less
; than nothing), out < base, and (base - out) * (quote + net) >= base * quote.
(set-logic QF_NIA)
(declare-const base Int) (declare-const quote Int) (declare-const amount Int) (declare-const k Int) (declare-const fee Int) (declare-const net Int) (declare-const out Int)
(assert (and (> base 0) (> quote 0) (> amount 0) (>= k 0) (< k 10000) (>= out 0)))
(assert (and (>= (* fee 10000) (* amount k)) (< (* (- fee 1) 10000) (* amount k))))
(assert (= net (- amount fee)))
(assert (<= (* out (+ quote net)) (* base net)))
(assert (< (* base net) (* (+ out 1) (+ quote net))))
(assert (not (and (<= 0 fee) (<= fee amount) (<= 0 net) (<= net amount) (< out base) (>= (* (- base out) (+ quote net)) (* base quote)))))
(check-sat)
