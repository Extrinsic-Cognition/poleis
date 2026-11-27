; A QUIET STRETCH IS ONE STEP (law.rs Branch::walk), over the unbounded integers. A whole step S > 0 from acc = 0 holding one spot h ends with
; acc = h * S, and the boundary's floor q of acc over S is h itself: a walked whole step takes its held spot, no division. When h is the
; observation (step_toward(obs, spot) = obs) the observation stays where it is and the sum gains obs * S; by induction from j = 0, with cj the
; sum after j such steps cum + j * obs * S, one more keeps it: k of them add k * obs * S, the skip's one addition. And from the start of a
; step (at < closes, at <= end, the boundaries at at + i * S for i >= 1), the loop applies boundary i exactly when at + i * S <= end and
; at + i * S < closes, which is exactly i <= k for k the floor of (last - at) over S, last the smaller of end and closes - 1: the skip applies
; the boundaries the loop would, and the first it leaves is not due.
(set-logic QF_NIA)
(declare-const obs Int) (declare-const h Int) (declare-const S Int) (declare-const acc Int) (declare-const q Int)
(declare-const cum Int) (declare-const j Int) (declare-const cj Int)
(declare-const at Int) (declare-const end Int) (declare-const closes Int) (declare-const last Int) (declare-const k Int) (declare-const i Int)
(assert (and (>= obs 0) (>= h 0) (> S 0) (>= cum 0)))
(assert (= acc (+ 0 (* h S))))
(assert (and (<= (* q S) acc) (< acc (* (+ q 1) S))))
(assert (and (>= j 0) (= cj (+ cum (* j obs S)))))
(assert (and (<= at end) (< at closes) (= last (ite (< end (- closes 1)) end (- closes 1)))))
(assert (and (<= (* k S) (- last at)) (< (- last at) (* (+ k 1) S))))
(assert (>= i 1))
(assert (not (and
  (= q h)
  (=> (= h obs) (and (= q obs) (= (+ cj (* obs S)) (+ cum (* (+ j 1) obs S)))))
  (>= k 0)
  (= (and (<= (+ at (* i S)) end) (< (+ at (* i S)) closes)) (<= i k)))))
(check-sat)
