; A TRADER CANNOT LOSE MORE THAN IT OPENED WITH, AND ONE THAT HOLDS WHAT IT OPENED WITH SCORES NOTHING (law.rs settle_of), over the unbounded integers.
; q is the trader's quote in the world that happened, g its claim marked at the receipts, B the budget, o the opening claim marked at the receipts;
; the result is q + g - B - o. Claim: the result is at least -(B + o), and when q = B and g = o it is 0.
(set-logic QF_LIA)
(declare-const q Int) (declare-const g Int) (declare-const B Int) (declare-const o Int) (declare-const result Int)
(assert (and (>= q 0) (>= g 0) (> B 0) (>= o 0)))
(assert (= result (- (+ q g) (+ B o))))
(assert (not (and (>= result (- (+ B o))) (=> (and (= q B) (= g o)) (= result 0)))))
(check-sat)
