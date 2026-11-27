; A PURSE CANNOT LOSE MORE THAN ITS DEPOSIT (law.rs settle_of and back_of), over the unbounded integers. A deposit d opens a purse with B of $WARM and
; C of the claim, B + C = d (a claim costs one $WARM to mint). x is C times the worth; the worth is at most PRICE, so x is at most PRICE * C; o is x
; floored by PRICE (worth_of): what the purse opened with of the claim, marked at the receipts. q and gw are what the purse holds at the settle, its
; $WARM and its claims marked: neither is negative. The result is q + gw - B - o. Claim: d + result is not negative, so what a road gives back to a
; balance (the deposit and the result) is never cut off at zero and nothing is forgiven. PRICE stands as its numeral, so the logic is linear.
(set-logic QF_LIA)
(declare-const d Int) (declare-const B Int) (declare-const C Int) (declare-const x Int) (declare-const o Int)
(declare-const q Int) (declare-const gw Int) (declare-const result Int)
(assert (and (>= B 0) (>= C 0) (= d (+ B C)) (>= q 0) (>= gw 0)))
(assert (and (>= x 0) (<= x (* 1000000000000 C))))
(assert (and (<= (* 1000000000000 o) x) (< x (* 1000000000000 (+ o 1)))))
(assert (= result (- (+ q gw) (+ B o))))
(assert (not (>= (+ d result) 0)))
(check-sat)
