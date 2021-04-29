

#ifndef RUST_DOT_H
#define RUST_DOT_H

#include <stdbool.h>
#include <stdlib.h>
#include "hand_index.h"

typedef void * rust_IndexerPtr;

// return a pointer to the allocated hand indexer to be used in other functions
void * rust_init_indexer (uint32_t rounds, const uint8_t cards_per_round[], bool *success);

void rust_free_indexer(void * indexer);

// assume 0 to be an error rather than a zero size i guess
uint64_t rust_indexer_size(void * v_indexer, uint32_t round);

uint64_t rust_index_all(void * v_indexer, const uint8_t cards[], uint64_t indices[]);

// false on error
bool rust_unindex(void * v_indexer, uint32_t round, uint64_t index, uint8_t cards[]);

// if we have extra cards this function just doesnt use them.
// if we don't even have enough cards for one round, we return 0.
//      - which doesn't work as a fail flag since an index can be 0
uint64_t rust_index_round(void * v_indexer, const uint8_t cards[], uint32_t ncards);

hand_indexer_state_t* rust_init_indexer_state(void * v_indexer);

void rust_free_state(void* v_state);

#endif
