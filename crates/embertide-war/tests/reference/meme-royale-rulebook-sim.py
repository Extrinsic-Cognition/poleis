#!/usr/bin/env python3
# Where this file came from: the brief of lane/the-war-is-its-own-program (claude-54dd2237, Nov 24 2026, its folder meme-royale/brief/), copied whole below this line: the owner's model, normative for embertide-war and poleis_wars.
# His war on Meme Royale's whole rulebook, with his half for deserters (his "Yes, run it", Nov 24 2026, to "Run your war on Meme Royale's
# whole rulebook, with your half?"). The rulebook: docs/meme-royale-the-program-read-back-hypertrace-2026-11-24.md (read from Meme Royale's
# own app, its live settings and its rebuilt bytecode). The war's ground stays pump.fun's: two houses priced in one coin beneath, the
# arithmetic of war-rules-sim.py beside this file (checked there against pump.fun's own program on 120 trades), loaded, not copied.
# What is Meme Royale's here: the raid opens by a stake, lasts by the coins' size, is won by beating a line that falls to nothing by the end,
# gives the side that lost a two-minute last stand, settles by the loser's pot buying the winner's coin for the loser's holders (stakers
# 10% better), and taxes the winners who sell right after. What is his: a deserter, or one who changes sides, leaves with half of what he
# brought, the rest to his house's pot (Meme Royale's own chicken tax does not stop the first deserter in this war: section 26 of the doc).
# Standard library only. usage: python3 -I meme-royale-rulebook-sim.py <out.json>
import importlib.util, json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location('wrs', os.path.join(HERE, 'war-rules-sim.py'))
wrs = importlib.util.module_from_spec(spec); spec.loader.exec_module(wrs)
SOL, VT, RT, N = wrs.SOL, wrs.VT, wrs.RT, wrs.N
HOUSES = ('crow', 'wolf')                                   # the crow attacks, the wolf defends
# Meme Royale's settings, read from its live global account (meme-royale-the-program-read-back-2026-11-24/global-2026-11-24.json)
MIN_RAID, MAX_RAID, RAID_STEP = 900, 21600, 900             # seconds
STAKE_OWN, STAKE_DELTA, STAKE_MAX = 0.05, 0.02, 0.20        # of the attacker's market cap; of the gap; the most that may be staked
LAST_STAND, REVIVAL = 2, 0.10                               # minutes; the ninth more a side in its last stand must climb
MIN_TAX, STAKER_BONUS = 0.10, 0.10
OPP_MIN, OPP_MAX = 0.5, 2.0
KEEP = 0.5                                                  # his half


def mcap(c):
    """Meme Royale's market cap: price times the coins sold (its own definition, liquidity x 4 on its curve), here on pump.fun's curve"""
    return c.vq * (c.rt0 - c.rt) // c.vt


def mcap_at(c, reserve):
    """the market cap this curve would have holding `reserve` of the coin beneath: vq = vq0 + reserve, vt = k / vq, mcap = vq x reserve / vq0"""
    vq0 = c.vq - c.rq
    return (vq0 + reserve) * reserve // vq0


def raid_minutes(w):
    """Meme Royale's length: 15 min to 6 h in 15-min steps, by the coins both houses have sold against twice a graduating supply"""
    sold = sum(w.h[h].rt0 - w.h[h].rt for h in HOUSES)
    return (MIN_RAID + sold * (MAX_RAID - MIN_RAID) // (2 * RT) // RAID_STEP * RAID_STEP) // 60


def fight(name, script, stand_reserve=None, burn=False, nested=True):
    """script(T) gives [(minute, kind, who, house)]: kind 'cross' (wolf to crow), 'desert', 'reinforce' (comes in with 1 SOL for `house`).
    stand_reserve: {house: n}, n fighters of that house who come in, 1 SOL each, in the first minute of its last stand, if it has one.
    burn: his ruling (Nov 24 2026, "Yes, burn them"): a deserter's coins, or the coins of one who changes sides, are burned, never sold into
    his house's curve; the house pays him out of its pool what they would fetch, at most half of what he brought, so leaving sinks his side
    only by what he is paid.
    nested: False prices both houses in SOL itself, no coin beneath: the war's own program as first built (lane/the-war-is-its-own-program,
    Nov 24 2026), whose tests hold this function's numbers. Only with burn."""
    assert nested or burn, 'the houses in SOL are run with his burn only'
    w = wrs.War(nested=nested); chest = {h: 0 for h in HOUSES}; took, brought, kind_of = {}, {}, {}; refused = []; burned = {h: 0 for h in HOUSES}
    worth_sol = (lambda x: wrs.sol_for(w.b, x)) if nested else (lambda x: x)        # what an amount of the houses' quote fetches in SOL
    def come_in(who, house, label):
        """a fighter comes in with 1 SOL; once the coin beneath's own curve is complete (about 85 SOL in all) nobody more can"""
        try: w.enter(who, house, SOL)
        except ValueError: refused.append(who); return
        brought[who] = SOL; kind_of[who] = label
    for i in range(N):
        for h in HOUSES:
            who = f'{h}{i}'; w.enter(who, h, SOL); brought[who] = SOL
    m0 = {h: mcap(w.h[h]) for h in HOUSES}
    in_range = OPP_MIN <= m0['wolf'] / m0['crow'] <= OPP_MAX
    # the stake: the crow's last ones in lock coins until they reach the larger of 5% of its market cap and 2% of the gap
    need = max(STAKE_OWN * m0['crow'], STAKE_DELTA * abs(m0['crow'] - m0['wolf']))
    price = w.h['crow'].vq / w.h['crow'].vt
    stakers, staked = [], 0
    for i in range(N - 1, 9, -1):
        if staked >= need: break
        who = f'crow{i}'; stakers.append(who); staked += w.tok[(who, 'crow')] * price
    assert staked <= STAKE_MAX * m0['crow'] + 1, 'the stake would pass the 20% cap'
    T = raid_minutes(w)

    def carry_out(who, house):
        if burn:
            t = w.tok[(who, house)]; c = w.h[house]
            q = t * c.vq // (c.vt + t) if t else 0
            worth = q - wrs.ceil_fee(q, wrs.PROTOCOL) - wrs.ceil_fee(q, wrs.CREATOR)
            lo, hi = 0, worth
            while lo < hi:
                mid = (lo + hi + 1) // 2
                if worth_sol(mid) <= KEEP * brought[who]: lo = mid
                else: hi = mid - 1
            lo = min(lo, c.rq)
            w.tok[(who, house)] = 0; burned[house] += t
            c.rq -= lo; c.vq -= lo                           # paid out of the pool: the price falls by that, and only by that
            if nested: w.coin[who] = w.coin.get(who, 0) + lo
            else: w.sol[who] = w.sol.get(who, 0) + lo
            return lo
        before = w.coin.get(who, 0)
        w.house_to_coin(who, house, w.tok[(who, house)])
        got = w.coin[who] - before
        lo, hi = 0, got
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if wrs.sol_for(w.b, mid) <= KEEP * brought[who]: lo = mid
            else: hi = mid - 1
        chest[house] += got - lo; w.coin[who] -= got - lo
        return lo

    def state(m):
        pool = sum(w.h[h].rq + chest[h] for h in HOUSES)   # both pools and both pots: what the winner takes
        out = {}
        for h in HOUSES:
            c = w.h[h]; now = mcap(c); line = mcap_at(c, pool)
            req = max(line - now, 0) * max(T - m, 0) / T
            out[h] = dict(now=now, line=line, req=req, delta=now - m0[h], meets=(now - m0[h]) > req)
        return out

    events = sorted(script(T)); timeline = []; winner = how = None; stand = None
    for m in range(0, T + LAST_STAND + 1):
        for (mm, kind, who, house) in events:
            if mm != m: continue
            if kind == 'reinforce':
                come_in(who, house, f'came in during the fight ({house})')
            elif kind == 'cross' and who not in stakers:
                carried = carry_out(who, 'wolf')
                if nested: w.coin_to_house(who, 'crow', carried)
                else: w.sol_to_house(who, 'crow', carried)
                kind_of[who] = 'changed sides (wolf to crow)'
            elif kind == 'desert' and who not in stakers:
                src = 'crow' if (who, 'crow') in w.tok and w.tok[(who, 'crow')] else 'wolf'
                carried = carry_out(who, src)
                took[who] = (w.coin_to_sol(who, carried) if nested else carried) / SOL if carried else 0.0; kind_of[who] = f'deserted ({src})'
        s = state(m)
        timeline.append({h: dict(delta=s[h]['delta'] / m0[h], req=s[h]['req'] / m0[h]) for h in HOUSES} | {'minute': m})
        if winner: break
        if stand:                                            # the side that lost climbs its line and a ninth more, or the other wins
            side, other, ends = stand
            if (stand_reserve or {}).get(side) and m == ends - LAST_STAND + 1:
                for k in range(stand_reserve[side]):
                    come_in(f'{side}-stand{k}', side, f'came in for the last stand ({side})')
                s = state(m); timeline[-1] = {h: dict(delta=s[h]['delta'] / m0[h], req=s[h]['req'] / m0[h]) for h in HOUSES} | {'minute': m}
            if s[side]['delta'] > s[side]['req'] / (1 - REVIVAL):
                winner, how = side, f'revived in its last stand at minute {m}'; break
            if m >= ends:
                winner, how = other, f'beat its line at minute {ends - LAST_STAND}; the last stand failed'; break
            continue
        meets = [h for h in HOUSES if s[h]['meets']]
        if m < T and len(meets) == 1:
            other = 'wolf' if meets[0] == 'crow' else 'crow'
            stand = (other, meets[0], m + LAST_STAND); continue
        if m < T and len(meets) == 2:
            g = {h: s[h]['delta'] / m0[h] for h in HOUSES}
            winner = 'crow' if g['crow'] > g['wolf'] else 'wolf'; how = f'both beat their lines at minute {m}; the further along won'; break
        if m >= T:
            g = {h: s[h]['delta'] / m0[h] for h in HOUSES}
            winner = 'crow' if g['crow'] > g['wolf'] else 'wolf'      # a tie goes to the defender
            how = 'the clock ran out; the further along won' + (' (a tie goes to the defender)' if g['crow'] == g['wolf'] else ''); break
    loser = 'wolf' if winner == 'crow' else 'crow'
    mcap_end = {h: mcap(w.h[h]) for h in HOUSES}
    # the settlement: the loser's pot buys the winner's coin on the winner's curve; the coins bought go to the loser's holders, stakers 10% better
    pot_in, cur = w.h[loser].rq + chest[loser], w.h[winner]
    lo, hi = 0, pot_in
    while lo < hi:
        mid = (lo + hi + 1) // 2
        raw = mid * 10000 // (10000 + wrs.PROTOCOL + wrs.CREATOR)
        if (raw - 1) * cur.vt // (cur.vq + raw - 1) <= cur.rt: lo = mid
        else: hi = mid - 1
    amount, quote, pf, cf = w._buy(cur, lo) if lo else (0, 0, 0, 0)
    (w._protocol_coin if nested else w._protocol_sol)(pf); w.creator_coin[winner] += cf; chest[winner] += pot_in - lo
    through = {who: t for (who, h), t in w.tok.items() if h == winner and t}
    holders = {who: t for (who, h), t in w.tok.items() if h == loser and t}
    weight = {who: t * (10 + round(10 * STAKER_BONUS)) if who in stakers else t * 10 for who, t in holders.items()}
    wsum = sum(weight.values()) or 1
    for who in holders:
        w.tok[(who, loser)] = 0; w.tok[(who, winner)] = w.tok.get((who, winner), 0) + amount * weight[who] // wsum
    w.h[loser].rq = 0; chest[loser] = 0
    every = {who: t for (who, h), t in w.tok.items() if h == winner and t}
    all_t = sum(every.values())
    paid, _, _ = w._sell(wrs.Curve(cur.vq, cur.vt, cur.rt), all_t)          # one price for every coin of the winner
    chest_t = sum(through.values())
    share = {who: paid * t // all_t + (chest[winner] * through.get(who, 0) // chest_t if chest_t else 0) for who, t in every.items()}
    total = sum(share.values())
    # the dump tax: the winners who sell at once pay the larger of 10% and the share of their market cap the raid added
    gain = max(0.0, (mcap_end[winner] - m0[winner]) / mcap_end[winner])
    dump_at_once = max(MIN_TAX, gain) if gain > 0 else 0.0
    # THE WINNER LIVES ON (his "It lives on", Nov 24 2026): nobody cashes out at the end. Each holder keeps his coins of the winner, and his
    # share of the winner's pot is paid to him in the coin beneath; he is valued three ways, the others holding: at the coins' price, selling
    # alone at once (paying the dump tax), and selling alone once the dump tax has faded.
    pb = w.b.vq / w.b.vt if nested else 1.0
    def alone(t, pot, tax):
        q = t * cur.vq // (cur.vt + t) if t else 0
        coins = q - wrs.ceil_fee(q, wrs.PROTOCOL) - wrs.ceil_fee(q, wrs.CREATOR) - int(q * tax) + pot
        if coins <= 0: return 0.0
        if not nested: return coins / SOL
        qb = coins * w.b.vq // (w.b.vt + coins)
        return (qb - wrs.ceil_fee(qb, wrs.PROTOCOL) - wrs.ceil_fee(qb, wrs.CREATOR)) / SOL
    lives = {}
    for who, t in every.items():
        pot = chest[winner] * through.get(who, 0) // chest_t if chest_t else 0
        lives[who] = dict(worth=(t * cur.vq / cur.vt + pot) * pb / SOL, alone_now=alone(t, pot, dump_at_once), alone_after=alone(t, pot, 0.0))
    for who, sol in took.items():                                              # the deserters already hold SOL
        if who not in lives: lives[who] = dict(worth=sol, alone_now=sol, alone_after=sol)
    # what backs a share of the coin that lives on: Meme Royale keeps the winner's pot in the coin ("additional liquidity on launch"), so the
    # winner's pool and its pot are everyone's by the coins each holds; valued as if all were sold together, so the shares add up to what is there
    backed_total = cur.rq + chest[winner]                                   # the whole pool: burned coins' backing stays with the holders
    if nested:
        qa, pfa, cfa = w._sell(wrs.Curve(w.b.vq, w.b.vt, w.b.rt), backed_total) if backed_total else (0, 0, 0)
        backed_sol = qa - pfa - cfa
    else:
        backed_sol = backed_total
    backed = {who: backed_sol * t / all_t / SOL for who, t in every.items()}
    per_fighter = {who: dict(holds=every.get(who, 0), backed_lamports=backed_sol * every.get(who, 0) // all_t if all_t else 0,
                             took_lamports=int(round(took.get(who, 0.0) * SOL)) if who in took else 0) for who in brought}
    settled = dict(pot_in=pot_in, bought_with=lo, coins_bought=amount, winner_reserve=cur.rq, winner_chest=chest[winner], all_held=all_t,
                   m0=m0, mcap_end=mcap_end, stake_need=need, stakers=stakers, minutes=T)
    for who, sol in took.items(): backed.setdefault(who, sol)
    if nested:
        q, pf2, cf2 = w._sell(w.b, total) if total else (0, 0, 0)
        net = q - pf2 - cf2
    else:
        net = total
    for who, c in share.items(): took[who] = net * c / total / SOL
    groups, groups_lives, owns = {}, {}, {}
    supply = sum(every.values()) or 1
    for who in brought:
        if who in kind_of: g = kind_of[who]
        elif who in stakers: g = 'staked (crow)'
        elif int(who[4:]) < 10: g = f'stayed, first ten in ({who[:4]})'
        else: g = f'stayed, last ten in ({who[:4]})'
        groups.setdefault(g, []).append(took.get(who, 0.0) * SOL / brought[who])
        owns[g] = owns.get(g, 0.0) + every.get(who, 0) / supply
        groups_lives.setdefault(g, []).append({k: v * SOL / brought[who] for k, v in (lives.get(who, dict(worth=0.0, alone_now=0.0, alone_after=0.0)) | {'backed': backed.get(who, 0.0)}).items()})
    return dict(name=name, minutes=T, winner=winner, how=how, opponents_in_range=in_range, stakers=len(stakers), stand_reserve=stand_reserve or {}, refused=len(refused),
                burn=burn, burned_share={h: burned[h] / max(1, w.h[h].rt0 - w.h[h].rt) for h in HOUSES},
                mcap_change={h: (mcap_end[h] - m0[h]) / m0[h] for h in HOUSES},
                stake_share_of_mcap=staked / m0['crow'], dump_tax_at_once=dump_at_once,
                groups={g: dict(mean=sum(v) / len(v), worst=min(v), best=max(v), n=len(v)) for g, v in groups.items()},
                lives_on={g: {k: dict(mean=sum(x[k] for x in v) / len(v), worst=min(x[k] for x in v), best=max(x[k] for x in v)) for k in ('worth', 'alone_now', 'alone_after', 'backed')} | {'n': len(v)}
                          for g, v in groups_lives.items()},
                owns_of_the_winner=owns, together=sum(took.values()), brought=sum(brought.values()) / SOL, timeline=timeline,
                per_fighter=per_fighter, settled=settled)


def base(T):
    """the war of sections 22 to 26, now on the clock: ten of the wolf's change sides in the first quarter, ten of the crow's desert in the second"""
    ev = [(1 + k * max(1, T // 4) // 10, 'cross', f'wolf{k}', 'crow') for k in range(10)]
    ev += [(T // 4 + k * max(1, T // 4) // 10, 'desert', f'crow{k}', 'crow') for k in range(10)]
    return ev
def spread(house, n, a, b, tag):
    return lambda T: [(int(a * T + k * (b - a) * T / n), 'reinforce', f'{tag}{k}', house) for k in range(n)]
SCENARIOS = [
    ('The war as before, now on the clock', base),
    ('Both sides send ten more, evenly', lambda T: base(T) + spread('crow', 10, 0, 1, 'crow-new')(T) + spread('wolf', 10, 0, 1, 'wolf-new')(T)),
    ('The crow pushes early, the wolf comes late', lambda T: base(T) + spread('crow', 10, 0, .25, 'crow-new')(T) + spread('wolf', 10, .9, 1, 'wolf-new')(T)),
    ('The crow rushes in twenty', lambda T: base(T) + spread('crow', 20, 0, .1, 'crow-new')(T)),
    ('The wolf answers twenty with twenty, later', lambda T: base(T) + spread('crow', 20, 0, .1, 'crow-new')(T) + spread('wolf', 20, .5, .6, 'wolf-new')(T)),
    ('A clean push: nobody deserts, the crow sends twenty in its first hour', lambda T: spread('crow', 20, 0, 60 / T, 'crow-new')(T)),
]
STANDS = [(f'A clean push of {p}, and the wolf answers {n} in its last stand', (lambda p: lambda T: spread('crow', p, 0, 60 / T, 'crow-new')(T))(p), {'wolf': n})
          for p in (10, 15) for n in (0, 5, 10, 15, 20, 25)]

if __name__ == '__main__':
    res = []
    runs = [(n + ', coins burned', s_, r_, True) for n, s_, *r_ in SCENARIOS + STANDS] + [(SCENARIOS[0][0] + ', coins sold', SCENARIOS[0][1], [], False)]
    for name, script, reserve, burn in runs:
        r = fight(name, script, reserve[0] if reserve else None, burn); res.append(r)
        print(f"{name:70s} {r['minutes']:4d} min | {r['winner']:4s} wins: {r['how']} | market cap crow {100 * r['mcap_change']['crow']:+.0f}% wolf {100 * r['mcap_change']['wolf']:+.0f}% | dump tax at once {100 * r['dump_tax_at_once']:.0f}% | together {r['together']:.2f} of {r['brought']:.0f}" + (f" | {r['refused']} could not come in" if r['refused'] else ''))
        for g, v in sorted(r['groups'].items()):
            L = r['lives_on'][g]
            print(f"    {g:40s} one lot {v['mean']:5.2f} | lives on: worth {L['worth']['mean']:5.2f} ({L['worth']['worst']:.2f} to {L['worth']['best']:.2f}), alone at once {L['alone_now']['mean']:5.2f}, alone after the dump tax {L['alone_after']['mean']:5.2f} ({v['n']})")
    json.dump(res, open(sys.argv[1], 'w'), indent=1)
    if len(sys.argv) > 2:                                                 # the program's reference: the houses in SOL, coins burned
        ref = []
        for name, script, *reserve in [SCENARIOS[0], SCENARIOS[-1], STANDS[3], STANDS[4]]:
            r = fight(name + ', in SOL', script, reserve[0] if reserve else None, True, nested=False)
            r.pop('timeline'); ref.append(r)
            print(f"IN SOL {r['name']:76s} {r['winner']} {r['how']} | " + ' | '.join(f"{g} {r['lives_on'][g]['backed']['mean']:.6f}" for g in sorted(r['lives_on'])))
        json.dump(ref, open(sys.argv[2], 'w'), indent=1)
