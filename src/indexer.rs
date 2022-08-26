mod rust_bindings;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use bitintr::Popcnt;
use rust_bindings::*;
use smallvec::{smallvec, SmallVec};

// TODO see if I need to use different types for cards
// and if a card buffer would make sense.

// TODO remember to change this if I update indexer.c
const MAX_INDICES: usize = 4;
pub type IndexVec<T> = SmallVec<[T; MAX_INDICES]>;
pub type CardVec<T> = SmallVec<[T; 8]>;
pub type IResult<T> = Result<T, IndexerError>;

#[derive(Debug, PartialEq)]
pub enum IndexerError {
    UnsupportedShape,
    DuplicateCards,
    CIndexOutOfRange,
    OutOfRange { index: usize, len: usize },
    WrongNumberOfCards,
    SomethingWentWrong,
    None,
}

#[derive(Debug)]
pub struct Indexer {
    pub shape: Vec<usize>,
    pub size: Vec<usize>,
    pub max_cards: usize,
    soul: IndexerPtr,
}

/// In an Indexer cards are represented as u8 (keeping true to
/// Kevins minimalist design).  Indexes are usize.  I've presumptuously
/// converted the original u64 values to usize to make it easier to use as
/// an index.  If that ruins your plans, sorry.
///
/// Shapes that produce indexes too large will panic.  I'm not
/// exactly sure what the cutoff is but it's in the 10s of billions
/// at least.  Luckily the holdem shape of 2, 3, 1, 1 ends up
/// with a maximum river index of 2428287420 which works on a 32 bit
/// machine.  (Upgrade your computer.)
///
/// A shape represents each of the independant card groups in the hand.
/// The order of cards within each group is not preserved.  The order of
/// card between groups is preserved.  The suit relations within each group
/// and between groups is preserved as you would expect.
/// For example AsKs | JdTs9s and KcAc | 9cJhTc would map to the same index.
impl Indexer {
    /// Return an indexer for the supplied shape.
    ///
    /// A shape is a list of cards per round.  For example
    /// [2, 3, 1, 1] would be holdem's shape.
    /// Return Error on unsupported shapes (the numbers are too big).
    pub fn new(shape: Vec<usize>) -> IResult<Self> {
        let rounds = shape.len() as u32;
        let shape_u8: Vec<u8> = shape.iter().map(|x| u8::try_from(*x).unwrap()).collect();
        let mut success = true;
        let soul =
            unsafe { rust_init_indexer(rounds, shape_u8.as_ptr(), &mut success as *mut bool) };

        if !success {
            return Err(IndexerError::UnsupportedShape);
        }
        let max_cards = shape.iter().sum::<usize>();
        let size = unsafe {
            // returns 0 on bad input or a zero sized indexer
            // both are stupid
            (0..shape.len())
                .map(|i| {
                    // May get None on 32 bit machine or similar problem.
                    usize::try_from(rust_indexer_size(soul, i as u32)).ok()
                })
                .collect::<Option<Vec<usize>>>()
        };

        let size = match size {
            Some(s) => s,
            None => return Err(IndexerError::UnsupportedShape),
        };

        Ok(Self {
            shape,
            size,
            max_cards,
            soul,
        })
    }

    /// Returns a Vector of indices.  One for the cards of each
    /// street.  You would use them to index into 4 different arrays.
    /// You must supply enough cards for the entire hand.  For example,
    /// if your indexer is of shape [2, 3] then you need 5 cards.

    /// # Argument
    ///
    /// `cards` - A u8 slice with cards from 0..52.
    ///
    /// # Return Error
    ///
    /// - if cards.len() does not exactly equal the sum of the shape
    /// - if there are duplicate cards
    ///
    /// # Examples
    /// ```
    /// use hand_indexer::Indexer;
    /// # use hand_indexer::IndexerError;
    /// let mut indexer = Indexer::new(vec![2, 5])?; // 2 hole cards, 5 boards cards
    /// let cards = vec![34, 23, 12, 51, 50, 33, 6];
    /// let indexes = indexer.index_all(&cards)?;
    /// //now you have indexes for the canonical 2 card and 7 card hand
    /// assert_eq!(indexes.len(), 2);
    /// assert!(indexes[0] < indexer.size[0]);
    /// assert!(indexes[1] < indexer.size[1]);
    /// # Ok::<(), IndexerError>(())
    /// ```
    pub fn index_all(&self, cards: &[u8]) -> IResult<IndexVec<usize>> {
        check_duplicates(cards)?;
        self.index_all_unchecked(cards)
    }

    /// Indexes one round and returns a single index.
    ///
    /// The round is determined by the number of cards.  For example
    /// an Indexer with shape [2, 3, 1, 1] and 5 cards will give you the
    /// index of the second round.  Extra cards that don't fit into the
    /// next round are ignored.

    /// # Argument
    ///
    /// `cards` - a u8 slice from 0..51.
    ///
    /// # Returns Error
    /// - if not enough cards for even the first round are supplied.
    /// - if there are duplicate cards
    ///
    /// # Examples
    /// ```
    /// use hand_indexer::Indexer;
    /// # use hand_indexer::IndexerError;
    /// let mut indexer = Indexer::new(vec![2, 3])?;
    /// let cards = vec![2, 6, 13, 11, 44];
    /// let hole_card_index = indexer.index_round(&cards[..2])?;
    /// let all_cards_index = indexer.index_round(&cards)?;
    /// assert_eq!(hole_card_index, indexer.index_all(&cards)?[0]);
    /// assert_eq!(all_cards_index, indexer.index_all(&cards)?[1]);
    /// assert!(hole_card_index < indexer.size[0]);
    /// assert!(all_cards_index < indexer.size[1]);
    /// # Ok::<(), IndexerError>(())
    /// ```
    pub fn index_round(&self, cards: &[u8]) -> IResult<usize> {
        check_duplicates(cards)?;
        // The c function just uses all the cards it can
        // and it's up to the caller to know what round it
        // it and what he's doing.  We'll return an error if the perfect
        // cards aren't supplied.
        let mut tot = 0;
        for round_cards in &self.shape {
            tot += round_cards;
            match tot.cmp(&cards.len()) {
                Ordering::Greater => return Err(IndexerError::WrongNumberOfCards),
                Ordering::Equal => break,
                Ordering::Less => (),
            }
        }
        if tot != cards.len() {
            return Err(IndexerError::WrongNumberOfCards);
        }
        self.index_round_unchecked(cards)
    }

    /// Returns the canonical cards from the index for a round.
    ///
    /// # Returns Error
    /// - if index is out of range for that round
    ///
    /// # Example
    /// ```
    /// use hand_indexer::Indexer;
    /// # use hand_indexer::IndexerError;
    /// let indexer = Indexer::new(vec![2, 3])?;
    /// let some_index = indexer.size[1] - 100; // the size is definately bigger than 100
    /// // we need to specify round 1.  this index would overflow round 0.
    /// let cards = indexer.unindex(some_index, 1)?;
    /// assert_eq!(indexer.index_round(&cards)?, some_index);
    /// # Ok::<(), IndexerError>(())
    /// ```
    pub fn unindex(&self, index: usize, round: usize) -> IResult<CardVec<u8>> {
        let num_cards = self.shape[..round + 1].iter().sum::<usize>();
        let mut cards: CardVec<u8> = smallvec![0; num_cards];

        unsafe {
            // now my cards should be filled
            if !rust_unindex(self.soul, round as u32, index as u64, cards.as_mut_ptr()) {
                return Err(IndexerError::CIndexOutOfRange);
            }
        }
        Ok(cards)
    }

    /// Like index_all but doesn't check for duplicates.
    /// It's left to the C code what will happen.
    pub fn index_all_unchecked(&self, cards: &[u8]) -> IResult<IndexVec<usize>> {
        // so we can check for success
        // *self.indexes_buffer.last_mut().unwrap() = 1;
        let mut indexes_buffer: [u64; MAX_INDICES] = [1; MAX_INDICES];

        if cards.len() != self.max_cards {
            return Err(IndexerError::WrongNumberOfCards);
        }
        let final_index =
            unsafe { rust_index_all(self.soul, cards.as_ptr(), indexes_buffer.as_mut_ptr()) };
        // Index all returns 0 on failure or last index on success.
        // I initialized buffer to 1s to test success.
        // (it should always succeed if indexer ptr is valid)
        if final_index != indexes_buffer[self.shape.len() - 1] {
            return Err(IndexerError::SomethingWentWrong);
        }
        // now my buffer should be filled
        Ok(indexes_buffer
            .iter()
            .take(self.shape.len())
            .map(|x| usize::try_from(*x).unwrap())
            .collect())
    }

    /// Like index_round but doesn't check for duplicates or for the correct
    /// number of cards.  Too few cards will return 0.  Too many will truncate
    /// the cards to an appropriate number and return the index for those.
    /// It's left to the C code what will happen in case of duplicates.
    pub fn index_round_unchecked(&self, cards: &[u8]) -> IResult<usize> {
        Ok(unsafe {
            rust_index_round(self.soul, cards.as_ptr(), cards.len() as u32)
                .try_into()
                .unwrap()
        })
    }

    /// Return a LazyIndexer
    pub fn incremental(&self) -> LazyIndexer {
        let state = unsafe { rust_init_indexer_state(self.soul) };
        LazyIndexer {
            soul: self.soul,
            state,
            shape: &self.shape,
            round: 0,
        }
    }
}

impl Drop for Indexer {
    fn drop(&mut self) {
        unsafe { rust_free_indexer(self.soul) }
    }
}

impl Clone for Indexer {
    fn clone(&self) -> Self {
        Self::new(self.shape.clone()).unwrap()
    }
}

pub struct LazyIndexer<'a> {
    soul: IndexerPtr,
    state: StatePtr,
    pub shape: &'a [usize],
    pub round: usize,
}

impl LazyIndexer<'_> {
    /// Returns the index for the next round.
    /// You must provide the exact number of cards for
    /// the current round and index the rounds one at a time.
    /// It is similar to unchecked in that it is left to the
    /// C code what happens if there are duplicate cards.
    ///
    /// # Return Error
    /// - if the incorrect number of cards are supplied
    /// - after it has indexed all rounds
    ///
    /// # Example
    /// ```
    /// use hand_indexer::Indexer;
    /// # use hand_indexer::IndexerError;
    /// let indexer = Indexer::new(vec![2, 3, 1, 1])?;
    /// let cards = vec![3, 45, 32, 22, 12, 11, 2];
    /// let mut lazy_i = indexer.incremental();
    /// assert_eq!(lazy_i.next_round(&cards[0..2])?, indexer.index_round(&cards[0..2])?);
    /// assert_eq!(lazy_i.next_round(&cards[2..5])?, indexer.index_round(&cards[0..5])?);
    /// assert_eq!(lazy_i.next_round(&cards[5..6])?, indexer.index_round(&cards[0..6])?);
    /// assert_eq!(lazy_i.next_round(&cards[6..7])?, indexer.index_round(&cards[0..7])?);
    /// assert_eq!(lazy_i.next_round(&cards[6..7]), Err(IndexerError::None));
    /// # Ok::<(), IndexerError>(())
    /// ```
    pub fn next_round(&mut self, cards: &[u8]) -> IResult<usize> {
        if self.round >= self.shape.len() {
            return Err(IndexerError::None);
        }
        if cards.len() != self.shape[self.round] {
            return Err(IndexerError::WrongNumberOfCards);
        }

        let index = unsafe {
            hand_index_next_round(self.soul, cards.as_ptr(), self.state)
                .try_into()
                .unwrap()
        };
        self.round += 1;
        Ok(index)
    }
}

impl Drop for LazyIndexer<'_> {
    fn drop(&mut self) {
        unsafe { rust_free_state(self.state) }
    }
}

// I'll leave this commented in case I want to go to
// conditional compilation.
// #[cfg(not(feature = "bitintr"))]
// fn duplicates(cards: &[u8]) -> bool {
//     let mut card_bits = 0_u64;
//     for c in cards {
//         // if c >= &52 {
//         //     panic!("oh no!");
//         // }
//         if (card_bits >> c) & 1 != 0 {
//             return true
//         }
//         card_bits |= 1 << c;
//     }
//     false
// }

// #[cfg(feature = "bitintr")]
fn check_duplicates(cards: &[u8]) -> IResult<()> {
    let mut card_bits = 0_u64;
    for c in cards {
        // Deciding whether to put this in.
        // if c >= &52 {
        //     panic!("oh no!");
        // }
        card_bits |= 1 << c;
    }
    if card_bits.popcnt() != cards.len() as u64 {
        Err(IndexerError::DuplicateCards)
    } else {
        Ok(())
    }
}

impl fmt::Display for IndexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfRange { index, len } => {
                write!(f, "the index was {} but the len was {}", index, len)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Error for IndexerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() -> IResult<()> {
        let indexer = Indexer::new(vec![2, 3])?;
        assert_eq!(indexer.shape, vec![2_usize, 3]);

        let indexer = Indexer::new(vec![2, 3, 1, 1])?;
        assert_eq!(indexer.shape, vec![2_usize, 3, 1, 1]);
        Ok(())
    }

    #[test]
    fn size() -> IResult<()> {
        let indexer = Indexer::new(vec![2, 3, 1, 1])?;
        assert_eq!(indexer.size[0], 169);
        assert_eq!(indexer.size[1], 1286792);
        assert_eq!(indexer.size[2], 55190538);
        assert_eq!(indexer.size[3], 2428287420);
        Ok(())
    }

    #[test]
    fn unindex() -> IResult<()> {
        let indexer = Indexer::new(vec![2, 3, 1, 1])?;
        assert_eq!(indexer.unindex(23, 0)?.len(), 2);
        assert_eq!(indexer.unindex(2345, 1)?.len(), 5);
        assert_eq!(indexer.unindex(345343, 2)?.len(), 6);
        Ok(())
    }

    #[test]
    fn index_all() -> IResult<()> {
        let indexer = Indexer::new(vec![2, 3, 1, 1])?;
        let cards = indexer.unindex(1234567, 3)?;
        assert_eq!(cards.len(), 7);
        assert_eq!(indexer.index_all(&cards)?[3], 1234567);
        Ok(())
    }

    #[test]
    fn index_round() -> IResult<()> {
        let indexer = Indexer::new(vec![2, 3, 1, 1])?;
        let cards: Vec<u8> = vec![45, 34, 32, 12, 11, 50, 2];
        let indexes = indexer.index_all(&cards)?;
        assert_eq!(indexer.index_round(&cards[..2])?, indexes[0]);
        assert_eq!(indexer.index_round(&cards[..5])?, indexes[1]);
        assert_eq!(indexer.index_round(&cards[..6])?, indexes[2]);
        assert_eq!(indexer.index_round(&cards[..7])?, indexes[3]);
        Ok(())
    }

    #[test]
    fn dups() -> IResult<()> {
        let indexer = Indexer::new(vec![7])?;
        assert_eq!(
            indexer.index_all(&[2, 7, 45, 38, 7, 44, 1]),
            Err(IndexerError::DuplicateCards)
        );
        Ok(())
    }
}
