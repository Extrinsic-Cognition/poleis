; THE DECISION IS THE AVERAGES' (law.rs decide), over the unbounded integers. The law compares the two sums, pass * 10000 > fail * (10000 + thr),
; without dividing. Claim: when it passes, PASS's true average (pass / span) exceeds FAIL's by the threshold exactly, so the floors shown beside it
; (tp = floor(pass / span), tf = floor(fail / span)) can differ from the verdict by rounding only: tp + 1 > tf * (10000 + thr) / 10000 - 1.
(set-logic QF_NIA)
(declare-const pass Int) (declare-const fail Int) (declare-const span Int) (declare-const thr Int) (declare-const tp Int) (declare-const tf Int)
(assert (and (>= pass 0) (>= fail 0) (> span 0) (>= thr 0) (<= thr 10000) (>= tp 0) (>= tf 0)))
(assert (and (<= (* tp span) pass) (< pass (* (+ tp 1) span))))
(assert (and (<= (* tf span) fail) (< fail (* (+ tf 1) span))))
(assert (> (* pass 10000) (* fail (+ 10000 thr))))
(assert (not (> (* (+ tp 1) 10000) (* tf (+ 10000 thr)))))
(check-sat)
