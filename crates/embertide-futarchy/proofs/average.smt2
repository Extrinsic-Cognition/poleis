; THE AVERAGE LIES BETWEEN THE LOWEST AND THE HIGHEST OBSERVATION (law.rs Branch::advance), by induction over the unbounded integers.
; At the opening span = 0 and cum = 0, where lo * span <= cum <= hi * span holds. The step: adding an observation between lo and hi for dt keeps it.
(set-logic QF_NIA)
(declare-const lo Int) (declare-const hi Int) (declare-const cum Int) (declare-const span Int) (declare-const obs Int) (declare-const dt Int)
(assert (and (>= span 0) (> dt 0) (<= lo obs) (<= obs hi)))
(assert (and (<= (* lo span) cum) (<= cum (* hi span))))
(assert (not (and (<= (* lo (+ span dt)) (+ cum (* obs dt))) (<= (+ cum (* obs dt)) (* hi (+ span dt))))))
(check-sat)
