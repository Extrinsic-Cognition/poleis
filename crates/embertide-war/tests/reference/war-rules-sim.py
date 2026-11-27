#!/usr/bin/env python3
# Where this file came from: the brief of lane/the-war-is-its-own-program (claude-54dd2237, Nov 24 2026, its folder meme-royale/brief/), copied whole below this line: the owner's model, normative for embertide-war and poleis_wars.
# The same war under other rules: which rule stops the first one to leave from taking the rest (the games doc's section 23).
#
# Step one is a check, not a result: the curve's arithmetic written out here in plain integers (from programs/pump/src/buy.rs:56-76,
# lib.rs:1991-2454, util/fee.rs:13-25 of ~/code-ec/pumpfun-source) must give, trade for trade, what pump.fun's own program gave when
# section 22's war ran on it (two-houses-on-pumpfun-2026-11-24.json). Only then is one rule changed at a time.
# Standard library only. usage: python3 -I war-rules-sim.py <the real run's json> <out.json>
import json, sys

SOL, COIN = 10**9, 10**6
VT, VQ_SOL, RT = 1_073_000_000_000_000, 30_000_000_000, 793_100_000_000_000
PROTOCOL, CREATOR, BUYBACK = 95, 30, 5000
ceil_fee = lambda x, bps: (x * bps + 9999) // 10000


class Curve:
    def __init__(self, vq, vt=VT, rt=RT):
        self.vt, self.vq, self.rt, self.rq, self.rt0 = vt, vq, rt, 0, rt
    def price(self): return self.vq / self.vt
    def full(self): return 1 - self.rt / self.rt0


def house_reserve(beneath, seed=VQ_SOL):
    """What the program gives a coin born priced in another coin (pump_quote.rs:101-123, 237-243): the SOL seed priced through its ancestor."""
    amount = RT * seed // (VT - RT)                                 # initial_amount of the root's seed
    amount = beneath.vt * amount // (beneath.vq + amount)           # convert through the ancestor's curve as it stands
    return amount * (VT - RT) // RT                                 # final_reserves


class War:
    """rules: fee_sink 'out' (pump.fun's wallets) or 'pool' (stays beneath for good); depth multiplies the curves' virtual reserve;
    toll is the share of SOL a leaver leaves in the pool; nested False prices each house in SOL with no coin beneath."""
    def __init__(self, fee_sink='out', depth=1, toll=0.0, nested=True):
        self.fee_sink, self.toll, self.nested = fee_sink, toll, nested
        self.b = Curve(VQ_SOL * depth)
        if nested:
            vq = house_reserve(self.b, VQ_SOL * depth)
            self.h = {'crow': Curve(vq), 'wolf': Curve(vq)}
        else:
            self.h = {'crow': Curve(VQ_SOL * depth), 'wolf': Curve(VQ_SOL * depth)}
        self.sol, self.coin, self.tok = {}, {}, {}                   # what each fighter holds
        self.out_sol = self.out_coin = 0                             # gone to outsiders (the protocol's wallets)
        self.creator_sol = 0; self.creator_coin = {'crow': 0, 'wolf': 0}
        self.locked_sol = 0; self.burned = 0                         # beneath for good
        self.trades = []

    # -- the four trades, as the program prices them --
    def _buy(self, curve, spend):
        raw = spend * 10000 // (10000 + PROTOCOL + CREATOR)
        pf, cf = ceil_fee(raw, PROTOCOL), ceil_fee(raw, CREATOR)
        total = raw + pf + cf
        quote = raw - (total - spend) if total > spend else raw
        amount = (quote - 1) * curve.vt // (curve.vq + quote - 1)
        if amount > curve.rt: raise ValueError('NotEnoughTokensToBuy')
        curve.vt -= amount; curve.rt -= amount; curve.vq += quote; curve.rq += quote
        return amount, quote, pf, cf

    def _sell(self, curve, amount):
        quote = amount * curve.vq // (curve.vt + amount)
        pf, cf = ceil_fee(quote, PROTOCOL), ceil_fee(quote, CREATOR)
        curve.vt += amount; curve.rt += amount; curve.vq -= quote; curve.rq -= quote
        return quote, pf, cf

    def _protocol_sol(self, pf):
        if self.fee_sink == 'pool': self.locked_sol += pf
        else: self.out_sol += pf
    def _protocol_coin(self, pf):
        if self.fee_sink == 'pool': self.burned += pf
        else: self.out_coin += pf

    def sol_to_coin(self, who, spend):
        amount, quote, pf, cf = self._buy(self.b, spend)
        self.sol[who] = self.sol.get(who, 0) - (quote + pf + cf); self.coin[who] = self.coin.get(who, 0) + amount
        self._protocol_sol(pf); self.creator_sol += cf
        self.trades.append(('buy beneath', amount, quote, pf, cf)); return amount

    def coin_to_sol(self, who, amount):
        quote, pf, cf = self._sell(self.b, amount)
        net = quote - pf - cf
        kept = int(net * self.toll); self.locked_sol += kept
        self.coin[who] -= amount; self.sol[who] = self.sol.get(who, 0) + net - kept
        self._protocol_sol(pf); self.creator_sol += cf
        self.trades.append(('sell beneath', amount, quote, pf, cf)); return net - kept

    def coin_to_house(self, who, house, spend):
        amount, quote, pf, cf = self._buy(self.h[house], spend)
        self.coin[who] -= quote + pf + cf; self.tok[(who, house)] = self.tok.get((who, house), 0) + amount
        self._protocol_coin(pf); self.creator_coin[house] += cf
        self.trades.append(('buy house', amount, quote, pf, cf)); return amount

    def house_to_coin(self, who, house, amount):
        quote, pf, cf = self._sell(self.h[house], amount)
        self.tok[(who, house)] -= amount; self.coin[who] = self.coin.get(who, 0) + quote - pf - cf
        self._protocol_coin(pf); self.creator_coin[house] += cf
        self.trades.append(('sell house', amount, quote, pf, cf)); return quote - pf - cf

    # -- the houses priced in SOL, no coin beneath --
    def sol_to_house(self, who, house, spend):
        amount, quote, pf, cf = self._buy(self.h[house], spend)
        self.sol[who] = self.sol.get(who, 0) - (quote + pf + cf); self.tok[(who, house)] = self.tok.get((who, house), 0) + amount
        self._protocol_sol(pf); self.creator_sol += cf; return amount
    def house_to_sol(self, who, house, amount):
        quote, pf, cf = self._sell(self.h[house], amount)
        net = quote - pf - cf; kept = int(net * self.toll); self.locked_sol += kept
        self.tok[(who, house)] -= amount; self.sol[who] = self.sol.get(who, 0) + net - kept
        self._protocol_sol(pf); self.creator_sol += cf; return net - kept

    # -- a fighter's moves --
    def enter(self, who, house, sol):
        if not self.nested: return self.sol_to_house(who, house, sol)
        before = self.coin.get(who, 0)
        self.sol_to_coin(who, sol)
        self.coin_to_house(who, house, self.coin[who] - before)
    def leave(self, who, house):
        if not self.nested: return self.house_to_sol(who, house, self.tok[(who, house)])
        self.house_to_coin(who, house, self.tok[(who, house)])
        return self.coin_to_sol(who, self.coin[who])
    def cross(self, who, src, dst):
        if not self.nested:
            got = self.house_to_sol(who, src, self.tok[(who, src)]); return self.sol_to_house(who, dst, got)
        before = self.coin.get(who, 0)
        self.house_to_coin(who, src, self.tok[(who, src)])
        self.coin_to_house(who, dst, self.coin[who] - before)
    def leave_together(self, group):
        """one price for all who leave in the same turn: their coins are sold as one lot and the proceeds shared by what each held"""
        took = {}
        for house in ('crow', 'wolf'):
            lot = [(w, self.tok.get((w, house), 0)) for w, hs in group if hs == house and self.tok.get((w, house), 0)]
            total = sum(t for _, t in lot)
            if not total: continue
            if self.nested:
                quote, pf, cf = self._sell(self.h[house], total); self._protocol_coin(pf); self.creator_coin[house] += cf
                for w, t in lot:
                    self.tok[(w, house)] = 0; self.coin[w] = self.coin.get(w, 0) + (quote - pf - cf) * t // total
            else:
                quote, pf, cf = self._sell(self.h[house], total); self._protocol_sol(pf); self.creator_sol += cf
                net = quote - pf - cf; kept = int(net * self.toll); self.locked_sol += kept
                for w, t in lot:
                    self.tok[(w, house)] = 0; took[w] = (net - kept) * t // total; self.sol[w] = self.sol.get(w, 0) + took[w]
        if self.nested:
            lot = [(w, self.coin.get(w, 0)) for w, _ in group if self.coin.get(w, 0)]
            total = sum(c for _, c in lot)
            if total:
                quote, pf, cf = self._sell(self.b, total); self._protocol_sol(pf); self.creator_sol += cf
                net = quote - pf - cf; kept = int(net * self.toll); self.locked_sol += kept
                for w, c in lot:
                    self.coin[w] = 0; took[w] = (net - kept) * c // total; self.sol[w] = self.sol.get(w, 0) + took[w]
        return [took.get(w, 0) for w, _ in group]

    def beneath_for_good(self):
        """SOL nobody can take any more: what was locked, and what stands behind burned coins"""
        if not self.nested: return self.locked_sol
        backing = 0
        if self.burned:                                              # what the burned coins would fetch if they were the last ones sold
            c = Curve(self.b.vq, self.b.vt, self.b.rt); backing = self.burned * c.vq // (c.vt + self.burned)
        return self.locked_sol + backing


N = 20
def the_war(rules, exit='in turn', lot=10):
    """forty come in with one SOL each, the houses in turn; ten of the wolf's cross to the crow; then all leave:
    the crow's first ten, then the others in the order they came. exit: 'in turn', 'one price a turn' (lots of `lot`), 'at the flag' (all at once)."""
    w = War(**rules)
    line = []
    for i in range(N):
        for house in ('crow', 'wolf'):
            who = f'{house}{i}'; w.enter(who, house, SOL); line.append((who, house))
    after_entry = dict(price=w.b.price() / (VQ_SOL * rules.get('depth', 1) / VT) if w.nested else w.h['crow'].price() / (VQ_SOL * rules.get('depth', 1) / VT),
                       crow=w.h['crow'].full(), wolf=w.h['wolf'].full(), sol_beneath=(w.b.rq if w.nested else w.h['crow'].rq + w.h['wolf'].rq) / SOL)
    held = (w.b.vq, w.b.vt, w.b.rt, w.b.rq)
    crossed = []
    for i in range(10):
        w.cross(f'wolf{i}', 'wolf', 'crow'); crossed.append(f'wolf{i}')
    after_fight = dict(crow=w.h['crow'].full(), wolf=w.h['wolf'].full(), beneath_untouched=held == (w.b.vq, w.b.vt, w.b.rt, w.b.rq))
    where = {who: house for who, house in line}
    for who in crossed: where[who] = 'crow'
    order = [(f'crow{i}', 'crow') for i in range(10)] + [(who, where[who]) for who, _ in line if not (who.startswith('crow') and int(who[4:]) < 10)]
    if exit == 'with what he brought':
        # a ledger, not the curve: each takes back his own SOL less the fees of the hops he made, whatever the others do
        hop_in, hop_out = lambda x: x * 10000 // (10000 + PROTOCOL + CREATOR), lambda x: x - ceil_fee(x, PROTOCOL) - ceil_fee(x, CREATOR)
        took = []
        for who, house in order:
            x = hop_in(hop_in(SOL))
            if who in crossed: x = hop_in(hop_out(x))
            took.append(hop_out(hop_out(x)))
    elif exit == 'in turn':
        took = [w.leave(who, house) for who, house in order]
    elif exit == 'one price a turn':
        took = []
        for k in range(0, len(order), lot): took += w.leave_together(order[k:k + lot])
    else:
        took = w.leave_together(order)
    took = [t / SOL for t in took]
    return dict(rules=rules, exit=exit, after_entry=after_entry, after_fight=after_fight, took=took, first_out=took[0], last_out=took[-1], best=max(took), worst=min(took),
                first_ten=took[:10], together=sum(took), out_sol=w.out_sol / SOL, out_coin=w.out_coin / COIN, creator_sol=w.creator_sol / SOL,
                creators_coin=(w.creator_coin['crow'] + w.creator_coin['wolf']) / COIN, beneath_for_good=w.beneath_for_good() / SOL,
                left_in_the_curve=(w.b.rq if w.nested else w.h['crow'].rq + w.h['wolf'].rq) / SOL, trades=len(w.trades))


def in_and_out(rules):
    w = War(**rules); w.enter('scout', 'crow', SOL); return w.leave('scout', 'crow') / SOL


def check(real):
    """the plain arithmetic against the program's own run: every trade of the war, and the two orders of leaving"""
    w = War()
    assert w.h['crow'].vq == real['birth']['crow']['virtual_quote'], (w.h['crow'].vq, real['birth']['crow']['virtual_quote'])
    log = [r for r in real['war_log'] if r.get('event')]
    crew = {'crow': [f'crow{i}' for i in range(N)], 'wolf': [f'wolf{i}' for i in range(N)]}
    for i in range(N):
        w.enter(crew['crow'][i], 'crow', SOL); w.enter(crew['wolf'][i], 'wolf', SOL)
    for i in range(10): w.cross(crew['wolf'][i], 'wolf', 'crow')
    took = [w.leave(crew['crow'][i], 'crow') / SOL for i in range(10)]
    bad = 0
    for k, (mine, theirs) in enumerate(zip(w.trades, log)):
        e = theirs['event']
        quote = e['sol_amount'] if mine[0].endswith('beneath') else e['quote_amount']
        if (mine[1], mine[2], mine[3], mine[4]) != (e['token_amount'], quote, e['protocol_fee'], e['creator_fee']):
            bad += 1
            if bad <= 4: print('  differs at trade', k, mine, (e['token_amount'], quote, e['protocol_fee'], e['creator_fee']))
    n = min(len(w.trades), len(log))
    gap = max(abs(a - b) for a, b in zip(took, real['ten_who_left']['each_took']))
    out = dict(trades_compared=n, trades_that_differ=bad, the_ten_largest_gap_sol=gap)
    for name, exit_order in (('first in, first out', 1), ('last in, first out', -1)):
        w = War(); line = []
        for i in range(N):
            for house in ('crow', 'wolf'): w.enter(f'{house}{i}', house, SOL); line.append((f'{house}{i}', house))
        back = [w.leave(who, house) / SOL for who, house in line[::exit_order]]
        theirs = real['everyone_leaves'][name]['sol_back_in_leaving_order']
        out[name] = dict(largest_gap_sol=max(abs(a - b) for a, b in zip(back, theirs)), together=sum(back), theirs=sum(theirs))
    out['in_and_out'] = dict(mine=in_and_out({}), theirs=real['round_trip']['sol_back'])
    return out


def sol_for(curve, coins):
    """what `coins` of the coin beneath would fetch in SOL, after the fees, without selling them"""
    quote = coins * curve.vq // (curve.vt + coins)
    return quote - ceil_fee(quote, PROTOCOL) - ceil_fee(quote, CREATOR)


def meme_royale(keep=None, siphon=0.0, losers_desert=False, runs_alone=None, made_up=False, convert=False, on_curve=False):
    """The same war on his formula (Nov 24 2026 01:1x UTC): the coins are required, and whoever deserts the fight loses.
    keep: 'chicken' is Meme Royale's own chicken tax, read from its program (meme-royale-the-program-read-back-hypertrace-2026-11-24.md):
    a seller during the fight pays the larger of 10% and the share of his house's price the fight added since it began, half to the house
    (out of the war) and half into his house's chest, the pot its winner takes.
    keep: the share of the 1 SOL he brought that a deserter may carry away (None: all his coins fetch, pump.fun's rules); what his coins
    fetch beyond it stays in his house's chest, for those who hold. A fighter who crosses to the other house deserts his own, on the same rule.
    siphon: at the flag, the share of the losing house's coins beneath that goes to the winning house's holders (Meme Royale's own rule,
    his Sept 9 2026 "the siphoning of liquidity"). The house that holds more of the coin beneath at the flag wins.
    After the flag every holder is paid in the coin beneath by the coins of his house he holds, and all sell it as one lot.
    losers_desert: before the flag, those still holding the house that is behind desert too, one after the other (what a holder does when
    staying to lose would leave him less than a deserter carries).
    runs_alone: one fighter who deserts by himself just before the flag, everyone else as in the script (the test of whether staying holds:
    what he ends with here against what he ends with when he stays).
    made_up: a deserter whose coins fetch less than he may carry is made up to it from his house's chest while it holds enough (the other
    reading of "half of what he brought": half whatever his coins sell for; without it he takes what his coins sell for, at most half).
    convert: Meme Royale's own end (memeroyale.xyz, July 2024, as BlockTempo of Jul 27 2024 and Solana Compass's listing tell it; its docs are
    gone): the winner absorbs ALL of the loser's liquidity, and the loser's coins are burned and converted into the winner's. Read here as the
    loser's whole pot buying the winner's house coin on the winner's own curve, the coins bought shared among the loser's holders by what each
    held; where the winner's curve runs out of coins to sell, the rest of the pot goes to the winner's chest.
    on_curve: what a deserter leaves behind (the chicken tax, as this seat reads it) stays in his house's pool ON its curve, raising the price of
    the coins still held, instead of in a chest beside it; so a later buy (the loser's conversion above all) pays the price the whole pool sets."""
    w = War(); chest = {'crow': 0, 'wolf': 0}; taxed = {'house': 0, 'pot': 0}; entry_price = {}
    for i in range(N):
        for house in ('crow', 'wolf'):
            w.enter(f'{house}{i}', house, SOL); entry_price[(f'{house}{i}', house)] = w.h[house].price()
    start_price = {h: w.h[h].price() for h in ('crow', 'wolf')}             # the fight begins once all forty are in
    def carry_out(who, house):
        """he sells his house's coins; returns the coins beneath he may carry, the rest goes to the house's chest"""
        before = w.coin.get(who, 0)
        price_now = w.h[house].price()
        w.house_to_coin(who, house, w.tok[(who, house)])
        got = w.coin[who] - before
        if keep is None: return got
        if keep in ('chicken', 'chicken since he joined'):
            ref = start_price[house] if keep == 'chicken' else entry_price.get((who, house), start_price[house])
            gain = max(0.0, (price_now - ref) / price_now)
            tax = int(got * max(0.10, gain))
            chest[house] += tax - tax // 2; taxed['house'] += tax // 2; taxed['pot'] += tax - tax // 2
            w.coin[who] -= tax
            return got - tax
        lo, hi = 0, got                                   # the most coins whose worth in SOL is within what he may carry
        while lo < hi:
            mid = (lo + hi + 1) // 2
            if sol_for(w.b, mid) <= keep * SOL: lo = mid
            else: hi = mid - 1
        if on_curve:
            w.h[house].vq += got - lo; w.h[house].rq += got - lo
        else:
            chest[house] += got - lo
        w.coin[who] -= got - lo
        if made_up and lo == got and sol_for(w.b, got) < keep * SOL:      # his coins fetched less than he may carry: the chest makes it up
            a, b = 0, chest[house]
            while a < b:
                mid = (a + b + 1) // 2
                if sol_for(w.b, got + mid) <= keep * SOL: a = mid
                else: b = mid - 1
            chest[house] -= a; w.coin[who] += a; lo = got + a
        return lo
    for i in range(10):                                   # ten of the wolf's cross to the crow
        who = f'wolf{i}'; carried = carry_out(who, 'wolf'); w.coin_to_house(who, 'crow', carried); entry_price[(who, 'crow')] = w.h['crow'].price()
    deserters = {}
    for i in range(10):                                   # the crow's first ten desert with SOL
        who = f'crow{i}'; carried = carry_out(who, 'crow')
        deserters[who] = w.coin_to_sol(who, carried) / SOL if carried else 0.0
    late = {}
    if losers_desert:
        behind = min(('crow', 'wolf'), key=lambda h: w.h[h].rq + chest[h])
        for who in [x for (x, h), t in list(w.tok.items()) if h == behind and t and x.startswith(behind)]:
            carried = carry_out(who, behind)
            late[who] = w.coin_to_sol(who, carried) / SOL if carried else 0.0
    if runs_alone:
        house = 'crow' if runs_alone.startswith('crow') else 'wolf'
        carried = carry_out(runs_alone, house)
        late[runs_alone] = w.coin_to_sol(runs_alone, carried) / SOL if carried else 0.0
    liq = {h: w.h[h].rq + chest[h] for h in ('crow', 'wolf')}
    winner = max(liq, key=liq.get); loser = 'wolf' if winner == 'crow' else 'crow'
    conversion = None
    if convert and liq[loser]:
        pot_in, cur = liq[loser], w.h[winner]
        lo, hi = 0, pot_in                                    # the most of the pot the winner's curve can take before it runs out of coins
        while lo < hi:
            mid = (lo + hi + 1) // 2
            raw = mid * 10000 // (10000 + PROTOCOL + CREATOR)
            if (raw - 1) * cur.vt // (cur.vq + raw - 1) <= cur.rt: lo = mid
            else: hi = mid - 1
        amount, quote, pf, cf = w._buy(cur, lo) if lo else (0, 0, 0, 0)
        w._protocol_coin(pf); w.creator_coin[winner] += cf
        chest[winner] += pot_in - lo
        through = {who: t for (who, h), t in w.tok.items() if h == winner and t}          # held the winner through the fight
        holders = {who: t for (who, h), t in w.tok.items() if h == loser and t}
        total_t = sum(holders.values())
        for who, t in holders.items():
            w.tok[(who, loser)] = 0; w.tok[(who, winner)] = w.tok.get((who, winner), 0) + amount * t // total_t
        w.h[loser].rq = 0; chest[loser] = 0
        conversion = dict(pot=pot_in / COIN, bought_with=lo / COIN, to_the_chest=(pot_in - lo) / COIN, coins=amount / COIN, curve_full=cur.full())
        every = {who: t for (who, h), t in w.tok.items() if h == winner and t}
        all_t = sum(every.values())
        c = Curve(cur.vq, cur.vt, cur.rt); paid, _, _ = w._sell(c, all_t)                # one price for every coin of the winner
        chest_t = sum(through.values())
        share = {who: paid * t // all_t + (chest[winner] * through.get(who, 0) // chest_t if chest_t else 0) for who, t in every.items()}
        conversion['paid_by_the_curve'] = paid / COIN; conversion['chest_to_those_who_held'] = chest[winner] / COIN
    if conversion is None:
        moved = int(liq[loser] * siphon)
        pot = {winner: liq[winner] + moved, loser: liq[loser] - moved}
        held = {h: sum(t for (who, hh), t in w.tok.items() if hh == h) for h in ('crow', 'wolf')}
        share = {}
        for (who, h), t in w.tok.items():
            if t and held[h]: share[who] = share.get(who, 0) + pot[h] * t // held[h]
    total = sum(share.values())
    quote, pf, cf = w._sell(w.b, total) if total else (0, 0, 0)        # all sell the coin beneath as one lot
    net = quote - pf - cf
    took = {who: net * c / total / SOL for who, c in share.items()}
    took.update(deserters); took.update(late)
    groups = {'deserted the crow': [f'crow{i}' for i in range(10)], 'crossed from the wolf to the crow': [f'wolf{i}' for i in range(10)],
              'stayed with the crow': [f'crow{i}' for i in range(10, N)], 'stayed with the wolf': [f'wolf{i}' for i in range(10, N)]}
    out = dict(keep=keep, siphon=siphon, losers_desert=losers_desert, convert=convert, conversion=conversion, took=took, chicken_tax={k: v / COIN for k, v in taxed.items()}, late_deserters=[round(v, 3) for v in late.values()], winner=winner, liquidity={h: liq[h] / COIN for h in liq}, chest={h: chest[h] / COIN for h in chest},
               groups={g: dict(mean=sum(took.get(x, 0) for x in xs) / len(xs), best=max(took.get(x, 0) for x in xs), worst=min(took.get(x, 0) for x in xs)) for g, xs in groups.items()},
               together=sum(took.values()))
    return out


def who_would_run(keep, siphon, made_up=False, convert=False, on_curve=False):
    """Does staying hold? For each of those who stay with the house that loses, and each of those who stay with the one that wins:
    what he ends with when he stays against what he ends with when he alone deserts just before the flag."""
    base = meme_royale(keep, siphon, made_up=made_up, convert=convert, on_curve=on_curve); out = {}
    for side, house in (('lost', 'wolf' if base['winner'] == 'crow' else 'crow'), ('won', base['winner'])):
        stay, run = [], []
        for i in range(10, N):
            who = f'{house}{i}'; stay.append(base['took'][who]); run.append(meme_royale(keep, siphon, runs_alone=who, made_up=made_up, convert=convert, on_curve=on_curve)['took'][who])
        out[side] = dict(would_rather_run=sum(1 for a, b in zip(stay, run) if b > a + 1e-9), of=len(stay), stays_mean=sum(stay) / len(stay),
                         stays_worst=min(stay), runs_mean=sum(run) / len(run), runs_worst=min(run), runs_best=max(run))
    return out


def tipping(keep, made_up=False):
    """The largest share of the loser's coins the winners can take with nobody on the losing side better off running alone, and the share at
    which the average of those who stay and lose ends level with what the average of them would carry by running alone."""
    def none_runs(sp): return who_would_run(keep, sp, made_up)['lost']['would_rather_run'] == 0
    lo, hi = 0.0, 1.0
    for _ in range(14):
        mid = (lo + hi) / 2
        if none_runs(mid): lo = mid
        else: hi = mid
    def level(sp):
        r = who_would_run(keep, sp, made_up)['lost']; return r['stays_mean'] >= r['runs_mean']
    a, b = 0.0, 1.0
    for _ in range(14):
        mid = (a + b) / 2
        if level(mid): a = mid
        else: b = mid
    return dict(nobody_runs_up_to=lo, the_average_is_level_at=a)


if __name__ == '__main__':
    real = json.load(open(sys.argv[1]))
    res = dict(check=check(real))
    print('the plain arithmetic against the program:', json.dumps(res['check'], indent=1))
    runs = [('pump.fun\'s rules', {}, 'in turn'),
            ('the fee goes down into the pool', dict(fee_sink='pool'), 'in turn'),
            ('one price a turn', {}, 'one price a turn'),
            ('nobody leaves before the flag', {}, 'at the flag'),
            ('a curve ten times deeper', dict(depth=10), 'in turn'),
            ('a toll of a fifth on leaving, kept in the pool', dict(toll=0.2), 'in turn'),
            ('the houses priced in SOL, no coin beneath', dict(nested=False), 'in turn'),
            ('the flag, and the fee goes down', dict(fee_sink='pool'), 'at the flag'),
            ('each leaves with what he brought', {}, 'with what he brought')]
    for d in (2, 3, 5, 30):
        runs.append((f'a curve {d} times deeper', dict(depth=d), 'in turn'))
    res['rules'] = []
    for name, rules, exit in runs:
        r = the_war(rules, exit); r['name'] = name; r['in_and_out'] = in_and_out(rules); res['rules'].append(r)
        print(f"{name:46s} first out {r['first_out']:6.2f} | last {r['last_out']:5.2f} | best {r['best']:6.2f} worst {r['worst']:5.2f} | together {r['together']:5.2f} | "
              f"price after 40 came in x{r['after_entry']['price']:.2f} | crow after the fight {100 * r['after_fight']['crow']:.0f}% | beneath for good {r['beneath_for_good']:.3f} | "
              f"to outsiders {r['out_sol']:.3f} SOL + {r['out_coin']:,.0f} coin | in and out {r['in_and_out']:.4f}")
    res['meme_royale'] = []
    for name in (('pump.fun\'s rules', None, 0.0), ('a deserter carries half of what he brought; the loser is siphoned whole', 0.5, 1.0),
                               ('a deserter carries nothing; the loser is siphoned whole', 0.0, 1.0), ('a deserter carries half; half of the loser is siphoned', 0.5, 0.5),
                               ('a deserter carries half; no siphon', 0.5, 0.0), ('pump.fun\'s rules, and the loser is siphoned whole', None, 1.0),
                               ('half; the loser siphoned whole; so the losing side deserts before the end', 0.5, 1.0, True),
                               ('half; two thirds of the loser siphoned', 0.5, 2 / 3), ('half; a third of the loser siphoned', 0.5, 1 / 3)):
        name, keep, siphon, ld = (name + (False,))[:4]
        r = meme_royale(keep, siphon, ld); r['name'] = name; res['meme_royale'].append(r)
        print(f"{name:74s} wins: {r['winner']} | " + ' | '.join(f"{g}: {v['mean']:.2f} ({v['worst']:.2f} to {v['best']:.2f})" for g, v in r['groups'].items()) + f" | together {r['together']:.2f}")
    for r in res['meme_royale']: r.pop('took', None)
    res['half'] = dict(tipping=tipping(0.5), tipping_made_up=tipping(0.5, True), who_would_run={}, who_would_run_made_up={},
                       the_losing_side_runs=meme_royale(0.5, 1.0, True), the_losing_side_runs_made_up=meme_royale(0.5, 1.0, True, made_up=True),
                       half_of_what_his_coins_sell_for={k: v for k, v in the_war(dict(toll=0.5), 'in turn').items() if k in ('first_out', 'last_out', 'best', 'worst', 'together')})
    for k in ('the_losing_side_runs', 'the_losing_side_runs_made_up'): res['half'][k].pop('took', None)
    for label, sp in (('all', 1.0), ('two thirds', 2 / 3), ('half', 0.5), ('a third', 1 / 3), ('nothing', 0.0)):
        r = who_would_run(0.5, sp); res['half']['who_would_run'][label] = r
        print(f"half; the winners take {label:10s} | of those who lose, would rather run: {r['lost']['would_rather_run']:2d} of {r['lost']['of']} (stays {r['lost']['stays_mean']:.2f}, least {r['lost']['stays_worst']:.2f}; runs alone {r['lost']['runs_mean']:.2f}, {r['lost']['runs_worst']:.2f} to {r['lost']['runs_best']:.2f})"
              f" | of those who win: {r['won']['would_rather_run']:2d} of {r['won']['of']} (stays {r['won']['stays_mean']:.2f}, least {r['won']['stays_worst']:.2f}; runs alone {r['won']['runs_mean']:.2f})")
        m = who_would_run(0.5, sp, True); res['half']['who_would_run_made_up'][label] = m
        print(f"      made up to half from the chest  | of those who lose, would rather run: {m['lost']['would_rather_run']:2d} of {m['lost']['of']} (stays {m['lost']['stays_mean']:.2f}, least {m['lost']['stays_worst']:.2f}; runs alone {m['lost']['runs_mean']:.2f}, {m['lost']['runs_worst']:.2f} to {m['lost']['runs_best']:.2f})")
    print('tipping:', res['half']['tipping'], '| made up:', res['half']['tipping_made_up'])
    print('the other reading, half of what his coins sell for:', res['half']['half_of_what_his_coins_sell_for'])
    res['meme_royale_as_it_was'] = {}
    for label, keep, oc in (("pump.fun's leaving", None, False), ('his half', 0.5, False), ('his half, the tax on the curve', 0.5, True),
                            ("Meme Royale's chicken tax", 'chicken', False), ("the chicken tax counted from his own entry", 'chicken since he joined', False)):
        conv = True
        r = meme_royale(keep, 0.0, convert=conv, on_curve=oc); r.pop('took', None); r['name'] = label
        run = who_would_run(keep, 0.0, convert=conv, on_curve=oc); r['who_would_run'] = run
        res['meme_royale_as_it_was'][label] = r
        print(f"MEME ROYALE'S END, {label:52s} " + ' | '.join(f"{g}: {v['mean']:.2f} ({v['worst']:.2f} to {v['best']:.2f})" for g, v in r['groups'].items())
              + f" | together {r['together']:.2f} | conversion {r['conversion']} | losers better off running {run['lost']['would_rather_run']} of {run['lost']['of']} (runs {run['lost']['runs_mean']:.2f}), winners {run['won']['would_rather_run']}")
    for k in ('the_losing_side_runs', 'the_losing_side_runs_made_up'):
        r = res['half'][k]; print(f"{k:30s} " + ' | '.join(f"{g}: {v['mean']:.2f} ({v['worst']:.2f} to {v['best']:.2f})" for g, v in r['groups'].items()) + f" | late: {r['late_deserters']}")
    json.dump(res, open(sys.argv[2], 'w'), indent=1)
