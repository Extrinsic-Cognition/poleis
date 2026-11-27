; THE POOL'S OWN MIX IS EVEN TO THE FEE (law.rs Terms::purse_of, traded), over the reals, before any floor. A pool opens with G of the claim and
; Q of $WARM; a purse opens with B of $WARM and C of the claim, its claims to its $WARM as the pool's are (C * Q = B * G; x = B / Q = C / G).
; The fee is $WARM beside the pool, a share 1 - phi of what a buy pays and of what a sell receives, phi in (0, 1]. Buying with all of B, only
; phi * B reaches the pool: it holds Q + phi B of $WARM and G Q / (Q + phi B) of the claim, so the price is up = (Q + phi B)^2 / (G Q): it is
; lifted by (1 + phi x)^2. Selling all of C, the pool trades all of it, and the fee comes out of what left: the pool holds G + C of the claim and
; Q G / (G + C) of $WARM, so the price is down = Q G / (G + C)^2: it is lowered by (1 + x)^2. Claim: those two factors, and their ratio
; (1 + phi x)^2 / (1 + x)^2 lies between phi^2 and 1: a seller of a purse's whole claim moves the price a little more than a buyer with its
; whole $WARM, by the fee and no more.
(set-logic QF_NRA)
(declare-const G Real) (declare-const Q Real) (declare-const B Real) (declare-const C Real) (declare-const x Real) (declare-const phi Real)
(declare-const up Real) (declare-const down Real)
(assert (and (> G 0) (> Q 0) (> B 0) (> C 0) (> phi 0) (<= phi 1)))
(assert (and (= (* C Q) (* B G)) (= (* x Q) B)))
(assert (= (* up G Q) (* (+ Q (* phi B)) (+ Q (* phi B)))))
(assert (= (* down (+ G C) (+ G C)) (* Q G)))
(assert (not (and
  (= (* up G) (* Q (+ 1 (* phi x)) (+ 1 (* phi x))))
  (= (* down G (+ 1 x) (+ 1 x)) Q)
  (<= (* phi phi (+ 1 x) (+ 1 x)) (* (+ 1 (* phi x)) (+ 1 (* phi x))))
  (<= (* (+ 1 (* phi x)) (+ 1 (* phi x))) (* (+ 1 x) (+ 1 x))))))
(check-sat)
