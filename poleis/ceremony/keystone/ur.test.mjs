// The reference's own vectors (ngraveio/bc-ur tests: rng.test.ts, fountain.test.ts, crc32.test.ts), read by ur.mjs.
import test from 'node:test';
import assert from 'node:assert/strict';
import { crc32, sha256, Xoshiro, chooseDegree, chooseFragments, cborEncode, cborDecode, tagged, urEncode, urParts, URDecoder, bytewordsDecode } from './ur.mjs';

const text = (s) => new TextEncoder().encode(s);
const hex = (h) => Uint8Array.from(h.match(/../g).map((x) => parseInt(x, 16)));
const u32be = (n) => Uint8Array.of(n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255);
const makeMessage = (len, seed = 'Wolf') => { const r = new Xoshiro(text(seed)); return Uint8Array.from({ length: len }, () => r.nextInt(0, 255)); };

test('sha-256 and crc-32 read as the world does', () => {
  assert.equal(Buffer.from(sha256(text('abc'))).toString('hex'), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  assert.equal(Buffer.from(sha256(new Uint8Array(1000).fill(97))).toString('hex'), '41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3');
  assert.equal(crc32(text('Hello, world!')).toString(16), 'ebe6c6e6');
  assert.equal(crc32(text('Wolf')).toString(16), '598c84dc');
  assert.equal(crc32(hex('d9012ca20150c7098580125e2ab0981253468b2dbc5202d8641947da')).toString(16), 'd22c52b6');
});

test('xoshiro256** gives the reference sequences', () => {
  const a = new Xoshiro(text('Wolf'));
  assert.deepEqual(Array.from({ length: 100 }, () => Number(a.next() % 100n)), [42, 81, 85, 8, 82, 84, 76, 73, 70, 88, 2, 74, 40, 48, 77, 54, 88, 7, 5, 88, 37, 25, 82, 13, 69, 59, 30, 39, 11, 82, 19, 99, 45, 87, 30, 15, 32, 22, 89, 44, 92, 77, 29, 78, 4, 92, 44, 68, 92, 69, 1, 42, 89, 50, 37, 84, 63, 34, 32, 3, 17, 62, 40, 98, 82, 89, 24, 43, 85, 39, 15, 3, 99, 29, 20, 42, 27, 10, 85, 66, 50, 35, 69, 70, 70, 74, 30, 13, 72, 54, 11, 5, 70, 55, 91, 52, 10, 43, 43, 52]);
  const b = new Xoshiro(u32be(crc32(text('Wolf'))));
  assert.deepEqual(Array.from({ length: 100 }, () => Number(b.next() % 100n)), [88, 44, 94, 74, 0, 99, 7, 77, 68, 35, 47, 78, 19, 21, 50, 15, 42, 36, 91, 11, 85, 39, 64, 22, 57, 11, 25, 12, 1, 91, 17, 75, 29, 47, 88, 11, 68, 58, 27, 65, 21, 54, 47, 54, 73, 83, 23, 58, 75, 27, 26, 15, 60, 36, 30, 21, 55, 57, 77, 76, 75, 47, 53, 76, 9, 91, 14, 69, 3, 95, 11, 73, 20, 99, 68, 61, 3, 98, 36, 98, 56, 65, 14, 80, 74, 57, 63, 68, 51, 56, 24, 39, 53, 80, 57, 51, 81, 3, 1, 30]);
  const c = new Xoshiro(text('Wolf'));
  assert.deepEqual(Array.from({ length: 100 }, () => c.nextInt(1, 10)), [6, 5, 8, 4, 10, 5, 7, 10, 4, 9, 10, 9, 7, 7, 1, 1, 2, 9, 9, 2, 6, 4, 5, 7, 8, 5, 4, 2, 3, 8, 7, 4, 5, 1, 10, 9, 3, 10, 2, 6, 8, 5, 7, 9, 3, 1, 5, 2, 7, 1, 4, 4, 4, 4, 9, 4, 5, 5, 6, 9, 5, 1, 2, 8, 3, 3, 2, 8, 4, 3, 2, 1, 10, 8, 9, 3, 10, 8, 5, 5, 6, 7, 10, 5, 8, 9, 4, 6, 4, 2, 10, 2, 1, 7, 9, 6, 7, 4, 2, 5]);
});

test('the fountain chooses the reference degrees and fragments', () => {
  assert.deepEqual(Array.from({ length: 200 }, (_, i) => chooseDegree(11, new Xoshiro(text(`Wolf-${i + 1}`)))), [11, 3, 6, 5, 2, 1, 2, 11, 1, 3, 9, 10, 10, 4, 2, 1, 1, 2, 1, 1, 5, 2, 4, 10, 3, 2, 1, 1, 3, 11, 2, 6, 2, 9, 9, 2, 6, 7, 2, 5, 2, 4, 3, 1, 6, 11, 2, 11, 3, 1, 6, 3, 1, 4, 5, 3, 6, 1, 1, 3, 1, 2, 2, 1, 4, 5, 1, 1, 9, 1, 1, 6, 4, 1, 5, 1, 2, 2, 3, 1, 1, 5, 2, 6, 1, 7, 11, 1, 8, 1, 5, 1, 1, 2, 2, 6, 4, 10, 1, 2, 5, 5, 5, 1, 1, 4, 1, 1, 1, 3, 5, 5, 5, 1, 4, 3, 3, 5, 1, 11, 3, 2, 8, 1, 2, 1, 1, 4, 5, 2, 1, 1, 1, 5, 6, 11, 10, 7, 4, 7, 1, 5, 3, 1, 1, 9, 1, 2, 5, 5, 2, 2, 3, 10, 1, 3, 2, 3, 3, 1, 1, 2, 1, 3, 2, 2, 1, 3, 8, 4, 1, 11, 6, 3, 1, 1, 1, 1, 1, 3, 1, 2, 1, 10, 1, 1, 8, 2, 7, 1, 2, 1, 9, 2, 10, 2, 1, 3, 4, 10]);
  const checksum = crc32(makeMessage(1024));
  assert.deepEqual(Array.from({ length: 30 }, (_, i) => chooseFragments(i + 1, 11, checksum).sort((x, y) => x - y)), [[0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [9], [2, 5, 6, 8, 9, 10], [8], [1, 5], [1], [0, 2, 4, 5, 8, 10], [5], [2], [2], [0, 1, 3, 4, 5, 7, 9, 10], [0, 1, 2, 3, 5, 6, 8, 9, 10], [0, 2, 4, 5, 7, 8, 9, 10], [3, 5], [4], [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], [0, 1, 3, 4, 5, 6, 7, 9, 10], [6], [5, 6], [7]]);
});

const PARTS = ["ur:bytes/1-9/lpadascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtdkgslpgh", "ur:bytes/2-9/lpaoascfadaxcywenbpljkhdcagwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsgmghhkhstlrdcxaefz", "ur:bytes/3-9/lpaxascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjksopdzmol", "ur:bytes/4-9/lpaaascfadaxcywenbpljkhdcasotkhemthydawydtaxneurlkosgwcekonertkbrlwmplssjtammdplolsbrdzcrtas", "ur:bytes/5-9/lpahascfadaxcywenbpljkhdcatbbdfmssrkzmcwnezelennjpfzbgmuktrhtejscktelgfpdlrkfyfwdajldejokbwf", "ur:bytes/6-9/lpamascfadaxcywenbpljkhdcackjlhkhybssklbwefectpfnbbectrljectpavyrolkzczcpkmwidmwoxkilghdsowp", "ur:bytes/7-9/lpatascfadaxcywenbpljkhdcavszmwnjkwtclrtvaynhpahrtoxmwvwatmedibkaegdosftvandiodagdhthtrlnnhy", "ur:bytes/8-9/lpayascfadaxcywenbpljkhdcadmsponkkbbhgsoltjntegepmttmoonftnbuoiyrehfrtsabzsttorodklubbuyaetk", "ur:bytes/9-9/lpasascfadaxcywenbpljkhdcajskecpmdckihdyhphfotjojtfmlnwmadspaxrkytbztpbauotbgtgtaeaevtgavtny", "ur:bytes/10-9/lpbkascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtwdkiplzs", "ur:bytes/11-9/lpbdascfadaxcywenbpljkhdcahelbknlkuejnbadmssfhfrdpsbiegecpasvssovlgeykssjykklronvsjkvetiiapk", "ur:bytes/12-9/lpbnascfadaxcywenbpljkhdcarllaluzmdmgstospeyiefmwejlwtpedamktksrvlcygmzemovovllarodtmtbnptrs", "ur:bytes/13-9/lpbtascfadaxcywenbpljkhdcamtkgtpknghchchyketwsvwgwfdhpgmgtylctotzopdrpayoschcmhplffziachrfgd", "ur:bytes/14-9/lpbaascfadaxcywenbpljkhdcapazewnvonnvdnsbyleynwtnsjkjndeoldydkbkdslgjkbbkortbelomueekgvstegt", "ur:bytes/15-9/lpbsascfadaxcywenbpljkhdcaynmhpddpzmversbdqdfyrehnqzlugmjzmnmtwmrouohtstgsbsahpawkditkckynwt", "ur:bytes/16-9/lpbeascfadaxcywenbpljkhdcawygekobamwtlihsnpalnsghenskkiynthdzotsimtojetprsttmukirlrsbtamjtpd", "ur:bytes/17-9/lpbyascfadaxcywenbpljkhdcamklgftaxykpewyrtqzhydntpnytyisincxmhtbceaykolduortotiaiaiafhiaoyce", "ur:bytes/18-9/lpbgascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtntwkbkwy", "ur:bytes/19-9/lpbwascfadaxcywenbpljkhdcadekicpaajootjzpsdrbalpeywllbdsnbinaerkurspbncxgslgftvtsrjtksplcpeo", "ur:bytes/20-9/lpbbascfadaxcywenbpljkhdcayapmrleeleaxpasfrtrdkncffwjyjzgyetdmlewtkpktgllepfrltataztksmhkbot"];

test('a one-part UR is the reference string', () => {
  assert.equal(urEncode('bytes', cborEncode(makeMessage(50))), "ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch");
  const d = new URDecoder();
  assert.ok(d.receive("ur:bytes/hdeymejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtgwdpfnsboxgwlbaawzuefywkdplrsrjynbvygabwjldapfcsdwkbrkch".toUpperCase()));
  assert.deepEqual(cborDecode(d.result), makeMessage(50));
});

test('the reference fountain decodes from its pure parts, in any order', () => {
  const d = new URDecoder();
  for (const p of [...PARTS.slice(0, 9)].reverse()) d.receive(p);
  assert.ok(d.done);
  assert.deepEqual(cborDecode(d.result), makeMessage(256));
});

test('missing pure parts are recovered from mixed ones', () => {
  const d = new URDecoder();
  for (const p of [PARTS[0], PARTS[2], PARTS[4], PARTS[6], PARTS[8], ...PARTS.slice(9)]) { if (d.receive(p)) break; }
  assert.ok(d.done, `progress ${d.progress}`);
  assert.deepEqual(cborDecode(d.result), makeMessage(256));
});

test('our own fountain parts decode, shuffled and repeated', () => {
  const msg = cborEncode(makeMessage(700, 'Poleis'));
  const parts = urParts('bytes', msg, 120);
  assert.ok(parts.length > 1);
  const order = [...parts, ...parts].map((p, i) => [((i * 7919) % 31), p]).sort(([a], [b]) => a - b).map(([, p]) => p);
  const d = new URDecoder();
  for (const p of order) d.receive(p);
  assert.deepEqual(d.result, msg);
});

test('cbor maps, tags and texts round-trip, and a broken checksum is refused', () => {
  const v = new Map([[2, Uint8Array.of(1, 2)], [1, tagged(37, Uint8Array.of(9))], [3, 'poleis'], [4, [44, true, 501, true]], [5, 4294967295]]);
  const back = cborDecode(cborEncode(v));
  assert.deepEqual([...back.keys()], [1, 2, 3, 4, 5]);
  assert.deepEqual(back.get(1), tagged(37, Uint8Array.of(9)));
  assert.equal(back.get(5), 4294967295);
  const good = urEncode('bytes', cborEncode(Uint8Array.of(1, 2, 3)));
  const body = good.slice('ur:bytes/'.length);
  assert.throws(() => bytewordsDecode(body.slice(0, -2) + (body.endsWith('ab') ? 'ad' : 'ab')), /checksum|word/);
});
