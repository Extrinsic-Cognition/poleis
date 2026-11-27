; A SPOT HELD INSIDE THE BAND LIES WITHIN max_step_bp OF THE OBSERVATION, BETWEEN THE OBSERVATION AND THE SPOT (law.rs step_toward), over the
; unbounded integers. d is the floor of obs * k / 10000; next is the spot clamped into obs - d .. obs + d: what a price counts for, each
; millisecond it stands, in the step's sum (law.rs Branch::advance, acc). proofs/stood.smt2 carries it from one held value to the boundary.
(set-logic QF_NIA)
(declare-const obs Int) (declare-const spot Int) (declare-const k Int) (declare-const d Int) (declare-const next Int)
(assert (and (> obs 0) (>= spot 0) (>= k 1) (<= k 10000) (>= d 0)))
(assert (<= (* d 10000) (* obs k)))
(assert (< (* obs k) (* (+ d 1) 10000)))
(assert (= next (ite (< spot (- obs d)) (- obs d) (ite (> spot (+ obs d)) (+ obs d) spot))))
(assert (not (and
  (<= (* 10000 (- next obs)) (* obs k))
  (<= (* 10000 (- obs next)) (* obs k))
  (<= (ite (< obs spot) obs spot) next)
  (<= next (ite (< obs spot) spot obs)))))
(check-sat)
