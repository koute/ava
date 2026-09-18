# Reports

A report is one document over the tournaments chosen for it: how every agent did against the tokens and the seconds it spent, over their finished rounds. The tournaments table on the tournaments page has a box beside every tournament, and the report button under it opens the report over the checked ones at `/report?tournament=<name>&tournament=<name>`. The page carries the styles and the fonts inside it and fetches nothing from the server, so saved from the browser it reads the same anywhere; the download link at its top hands it over as `report-<name>-<name>.html` straight away. A report of tournaments still playing is what their finished rounds hold and grows with them, so a report worth keeping is one saved once the tournaments are over.

An agent is a harness on a model, as the ratings key it, so two seats holding the same agent, in one tournament or across the chosen ones, are one row and one line. Only pairings between different agents count for the rounds and the score. The dollars are the tokens at the prices of the registry, so a run on a route without a price has none, and the ratios over dollars run over the runs that have one, with the count of the others behind the hover. The analyses of the runs are the analyst's tokens, not the agent's, and stay out.

## What it holds

Six tiles count what the report spans: the tournaments, their finished rounds, the runs, the dollars, the tokens through the backends and the run time. Three charts follow, drawn like the charts of a tournament page and hovering the same way:

- Passes over the budget, one stepped line per agent: the share of its runs that had a passing push by every share of the budget, so a line climbing early belongs to an agent that passes fast and a line staying low to one that passes rarely. The budget is the share of the seconds a run was given, which is what lets tournaments of different lengths share the chart.
- Score against dollars and score against output tokens, one mark per agent: the share of its rounds won against what one of its runs cost, in dollars and in thousands of output tokens. The marks towards the top left won cheaply.

Two tables over every agent follow:

- Cost: the runs, the runs that passed, the rounds against other agents as won-drawn-lost and the share of them won, the dollars and what they come to per run, per passing run and per round won, a draw counting half, and the tokens by kind: output, output per round won, input not read from the cache, and read from it.
- Time: the median share of the budget spent when the first push passed, over the runs that passed, and when the entry of record was pushed, over the runs that kept one; the seconds of the runs over the seconds they were given; the seconds spent inside requests to the backend over the seconds of the runs, which tells a slow backend from a slow harness and passes the whole when a harness had requests in flight at once; the mean seconds to the first token; the tokens and the requests a minute; the largest share of its window a run reached; the compactions; the model calls answered with an error, with the ones cut short upstream and the ones abandoned behind the hover.
Both tables arrive sorted by score, the cheapest run first among equals, and their headers sort them by any column, a blank cell last either way, the way the tables of the interface do. Every header explains its column behind the hover, and the medians leave out the runs that never passed or banked rather than counting them at the end of the budget, which is what the pass share beside them is for.
