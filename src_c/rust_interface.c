#include "rust.h"

// return a pointer to the allocated hand indexer
void * rust_init_indexer (uint32_t rounds, const uint8_t cards_per_round[], bool *success) {

    hand_indexer_t* indexer = (hand_indexer_t *) malloc(sizeof(hand_indexer_t));
    if (!indexer) {
        *success = false;
        return NULL;
    }
    if (!hand_indexer_init((uint_fast32_t) rounds, cards_per_round, indexer)){
        *success = false;
        return NULL;
    }
    *success = true;
    return indexer;
}


void rust_free_indexer(void * indexer) {
    hand_indexer_free((hand_indexer_t *) indexer);
    free(indexer);
}

// assume 0 to be an error rather than a zero size i guess
uint64_t rust_indexer_size(void * v_indexer, uint32_t round) {
    hand_indexer_t * indexer = (hand_indexer_t *) v_indexer;
    if (round >= indexer->rounds) {
        return 0;
    }
    return hand_indexer_size(indexer, (uint_fast32_t) round);
}


uint64_t rust_index_all(void * v_indexer, const uint8_t cards[], uint64_t indices[]) {
    hand_indexer_t * indexer = (hand_indexer_t *) v_indexer;

    return hand_index_all(indexer, cards, indices);
}

// false on error
bool rust_unindex(void * v_indexer, uint32_t round, uint64_t index, uint8_t cards[]) {
    hand_indexer_t * indexer = (hand_indexer_t *) v_indexer;
    // hand_unindex checks bounds for once
    return hand_unindex(indexer, (uint_fast32_t) round, index, cards);
}

// if we have extra cards this function just doesnt use them.
// if we don't even have enough cards for one round, we return 0.
//      - which doesn't work as a fail flag since an index can be 0
uint64_t rust_index_round(void * v_indexer, const uint8_t cards[], uint32_t ncards) {
    hand_indexer_t * indexer = (hand_indexer_t *) v_indexer;
    hand_indexer_state_t state;
    uint64_t index = 0;
    uint32_t cards_used = 0;

    hand_indexer_state_init(indexer, &state);

    // index rounds one at a time, till we run out of cards or rounds
    for (uint_fast32_t round = 0; round < indexer->rounds; round++) {
        cards_used += indexer->cards_per_round[round];
        if (cards_used > ncards) {
            break;
        }
        index = hand_index_next_round(indexer, cards, &state);
        // Hand_index_next_round takes the cards for that round only.
        // so we move the pointer forward
        cards += indexer->cards_per_round[round];
    }
    return index;
}

hand_indexer_state_t* rust_init_indexer_state(void * v_indexer) {
    hand_indexer_state_t * state = (hand_indexer_state_t*) malloc(sizeof(hand_indexer_state_t));
    // You don't need to coerce the v_indexer because it isn't used in this function.
    hand_indexer_state_init(v_indexer, state);
    return state;
}

void rust_free_state(void * v_state) {
    free(v_state);
}

