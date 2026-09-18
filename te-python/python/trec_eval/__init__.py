"""
`trec_eval` Python interface powered by `te-rust`.
"""

from typing import NamedTuple

ScoredDoc = NamedTuple('ScoredDoc', [('query_id', str), ('doc_id', str), ('score', float)])
Qrel = NamedTuple('Qrel', [('query_id', str), ('doc_id', str), ('relevance', int)])

__all__ = [
    "ScoredDoc",
    "Qrel",
]
