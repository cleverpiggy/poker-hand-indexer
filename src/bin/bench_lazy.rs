use std::time::{Duration, Instant};
extern crate hand_indexer;
use hand_indexer::{IndexVec, Indexer};
use smallvec::smallvec;
// compare indexing the rounds one at a time with lazy indexer
// to index all.
// in both cases we return a newly allocated vector

// with allocating in lazy (have since chanced the code)
// all: 64.286695ms
// one: 51.58235ms
// two: 61.682157ms
// three: 77.765627ms
// four: 81.283188ms

// with reusing buffer in lazy (how the code currently stands)
// all: 63.520378ms
// one: 33.405279ms
// two: 41.17406ms
// three: 52.176466ms
// four: 63.132026ms

fn main() {
    let indexer = Indexer::new(vec![2, 3, 1, 1]).unwrap();
    let mut cards = vec![0; 7];
    let mut all = Duration::new(0, 0);
    let mut one = Duration::new(0, 0);
    let mut two = Duration::new(0, 0);
    let mut three = Duration::new(0, 0);
    let mut four = Duration::new(0, 0);

    let mut buf: IndexVec<usize> = smallvec![0_usize; 4];

    for a in 0..6 {
        cards[0] = a as u8;
        for b in 6..12 {
            cards[1] = b as u8;
            for c in 12..18 {
                cards[2] = c as u8;
                for d in 18..26 {
                    cards[3] = d as u8;
                    for e in 26..32 {
                        cards[4] = e as u8;
                        for f in 32..38 {
                            cards[5] = f as u8;
                            for g in 38..44 {
                                cards[6] = g as u8;
                                let (result, time) = time_all(&indexer, &cards);
                                all += time;
                                one += time_lazy(&indexer, &cards, 1, &mut buf);
                                two += time_lazy(&indexer, &cards, 2, &mut buf);
                                three += time_lazy(&indexer, &cards, 3, &mut buf);
                                four += time_lazy(&indexer, &cards, 4, &mut buf);
                                assert_eq!(result, buf);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("all: {:?}", all);
    println!("one: {:?}", one);
    println!("two: {:?}", two);
    println!("three: {:?}", three);
    println!("four: {:?}", four);
}

fn time_all(indexer: &Indexer, cards: &[u8]) -> (IndexVec<usize>, Duration) {
    let start = Instant::now();
    let result = indexer.index_all(cards).unwrap();
    let time = Instant::now() - start;
    (result, time)
}

fn time_lazy(indexer: &Indexer, cards: &[u8], n: usize, result: &mut [usize]) -> Duration {
    const STARTS: [usize; 4] = [0, 2, 5, 6];
    const STOPS: [usize; 4] = [2, 5, 6, 7];
    let start = Instant::now();
    // let mut result: Vec<usize> = Vec::with_capacity(n);
    let mut lazi_i = indexer.incremental();
    for i in 0..n {
        result[i] = lazi_i.next_round(&cards[STARTS[i]..STOPS[i]]).unwrap();
    }
    let time = Instant::now() - start;
    time
}
