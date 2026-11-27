; BUYING AND SELLING STRAIGHT BACK NEVER PAYS, WITH THE FEE OF EACH LEG (no $WARM is made from the floors), over the unbounded integers.
; A buy of `amount` pays fee1 (0 <= fee1 <= amount) beside the pool; its net n1 takes `got` claims out, and the pool is (base - got, quote + n1).
; Selling `got` back trades all of them: gross = floor((quote + n1) * got / (base - got + got)), and the purse gets back = gross - fee2,
; 0 <= fee2 <= gross. Claim: back <= n1 - fee2 (a round trip gives back at most the net of its first leg less the second fee), so back <= amount.
(set-logic QF_NIA)
(declare-const base Int) (declare-const quote Int) (declare-const amount Int) (declare-const got Int) (declare-const gross Int) (declare-const back Int)
(declare-const fee1 Int) (declare-const fee2 Int) (declare-const n1 Int)
(assert (and (> base 0) (> quote 0) (> amount 0) (>= got 0) (>= gross 0)))
(assert (and (<= 0 fee1) (<= fee1 amount) (= n1 (- amount fee1))))
(assert (<= (* got (+ quote n1)) (* base n1)))
(assert (< (* base n1) (* (+ got 1) (+ quote n1))))
(assert (<= (* gross (+ (- base got) got)) (* (+ quote n1) got)))
(assert (< (* (+ quote n1) got) (* (+ gross 1) (+ (- base got) got))))
(assert (and (<= 0 fee2) (<= fee2 gross) (= back (- gross fee2))))
(assert (not (and (<= back (- n1 fee2)) (<= back amount))))
(check-sat)
