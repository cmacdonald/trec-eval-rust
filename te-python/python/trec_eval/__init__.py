"""
`trec_eval` Python interface powered by `te-rust`.
"""

from typing import Any, Iterable, List, NamedTuple, Optional, Set, Union

from trec_eval._trec_eval import EvalResult, Evaluator, __version__

ScoredDoc = NamedTuple('ScoredDoc', [('query_id', str), ('doc_id', str), ('score', float)])
Qrel = NamedTuple('Qrel', [('query_id', str), ('doc_id', str), ('relevance', int)])


def evaluate(
    qrels: Any,
    run: Any,
    measures: Optional[Union[str, Iterable[str], Set[str], List[str]]] = None,
    relevance_level: int = 1,
    complete_set_average: bool = False,
    judged_docs_only: bool = False,
    max_docs_per_topic: Optional[int] = None,
    num_docs_in_coll: int = 0,
    global_gains: Optional[str] = None,
) -> EvalResult:
    """Evaluate a run against relevance judgments using trec_eval.

    Parameters
    ----------
    qrels : str, os.PathLike, Dict[str, Dict[str, int]], or Iterable[Tuple[str, str, int]]
        Relevance judgments (file path, nested dict, or row iterable).
    run : str, os.PathLike, Dict[str, Dict[str, float]], or Iterable[Tuple[str, str, float]]
        System run results (file path, nested dict, or row iterable).
    measures : str or list of str, optional
        Measure names or groups (e.g. ['map', 'ndcg@10', 'P.5,10']). Defaults to 'official'.
    relevance_level : int, default 1
        Minimum relevance level at which a document is considered relevant (-l).
    complete_set_average : bool, default False
        Average over all queries in qrels instead of only retrieved ones (-c).
    judged_docs_only : bool, default False
        Remove all unjudged documents before evaluating (-J).
    max_docs_per_topic : int, optional
        Maximum number of documents per topic to evaluate (-M).
    num_docs_in_coll : int, default 0
        Number of documents in the collection (-N).
    global_gains : str, optional
        Global relevance-to-gain mapping (e.g. '1=3.5,2=9.0').

    Returns
    -------
    EvalResult
        Rich evaluation result with mapping protocol and .aggregate() / .per_query() methods.
    """
    evaluator = Evaluator(
        qrels=qrels,
        measures=measures,
        relevance_level=relevance_level,
        complete_set_average=complete_set_average,
        judged_docs_only=judged_docs_only,
        max_docs_per_topic=max_docs_per_topic,
        num_docs_in_coll=num_docs_in_coll,
        global_gains=global_gains,
    )
    return evaluator.evaluate(run)


__all__ = [
    "__version__",
    "Evaluator",
    "EvalResult",
    "ScoredDoc",
    "Qrel",
    "evaluate",
]

