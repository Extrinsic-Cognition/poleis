; WHAT THE TRADERS WIN THE POOL LOSES, TO THE ROUNDING (law.rs settle_of and pool_result), over the unbounded integers. One trader against the pool:
; a is the trader's claim times the worth, b the pool's base times the worth, c the opening claim times the worth, d the opening pool base times the
; worth, and the claim is conserved, a + b = c + d. Each is floored by the law's PRICE (worth_of): fa, fb, fc, fd. The quote parts cancel exactly, so
; this is the whole rounding. Claim: (fa + fb) - (fc + fd) lies within -1 and 1. PRICE stands as its numeral, 1000000000000, so the logic is linear.
(set-logic QF_LIA)
(declare-const a Int) (declare-const b Int) (declare-const c Int) (declare-const d Int)
(declare-const fa Int) (declare-const fb Int) (declare-const fc Int) (declare-const fd Int)
(assert (and (>= a 0) (>= b 0) (>= c 0) (>= d 0) (= (+ a b) (+ c d))))
(assert (and (<= (* 1000000000000 fa) a) (< a (* 1000000000000 (+ fa 1)))))
(assert (and (<= (* 1000000000000 fb) b) (< b (* 1000000000000 (+ fb 1)))))
(assert (and (<= (* 1000000000000 fc) c) (< c (* 1000000000000 (+ fc 1)))))
(assert (and (<= (* 1000000000000 fd) d) (< d (* 1000000000000 (+ fd 1)))))
(assert (not (and (<= (- 1) (- (+ fa fb) (+ fc fd))) (<= (- (+ fa fb) (+ fc fd)) 1))))
(check-sat)
