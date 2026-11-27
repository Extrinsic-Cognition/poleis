; THE OBSERVATION A BOUNDARY TAKES LIES INSIDE THE BAND (law.rs Branch::advance), by induction over the unbounded integers, as average.smt2 does it.
; lo and hi are the band's edges as step.smt2 leaves every held value: lo the larger of obs - d and the lower of obs and the step's lowest spot,
; hi the smaller of obs + d and the higher of obs and its highest spot. At the step's start acc = 0 and e = 0, where lo * e <= acc <= hi * e holds.
; The step (the line `self.acc = self.acc.saturating_add(step_toward(self.obs, spot, t.max_step_bp).saturating_mul(dt))`): a held value h
; between lo and hi, standing dt more milliseconds, keeps it. The boundary (the line `self.obs = self.acc / (t.step_ms as u128).max(1)`): with e
; the whole step S > 0, the floor q of acc over S lies between lo and hi. So the observation moves by at most max_step_bp of itself at a boundary
; and never leaves what stood.
(set-logic QF_NIA)
(declare-const lo Int) (declare-const hi Int) (declare-const acc Int) (declare-const e Int) (declare-const h Int) (declare-const dt Int)
(declare-const s Int) (declare-const S Int) (declare-const q Int)
(assert (and (>= e 0) (> dt 0) (<= lo h) (<= h hi)))
(assert (and (<= (* lo e) acc) (<= acc (* hi e))))
(assert (and (> S 0) (<= (* lo S) s) (<= s (* hi S))))
(assert (and (<= (* q S) s) (< s (* (+ q 1) S))))
(assert (not (and
  (<= (* lo (+ e dt)) (+ acc (* h dt)))
  (<= (+ acc (* h dt)) (* hi (+ e dt)))
  (<= lo q)
  (<= q hi))))
(check-sat)
