# label-tracker

Fork of <https://git.eno.space/label-tracker.git>.

Fetcher TODO:

Nightly polling, fetching:

1. fetch HEADs of new/updated tracked branches
1. create graph of those
    1. For each pair of branch heads, find relation
        - A ancestor of B -> A are on same release (A possibly a channel) (not technically true necessarily, might be freshly cut release as well)
        - A and B have common ancestor C -> A and B are on different releases
1. fetch pull requests newly merged {number, merge_commit}
    1. for each such HEAD,

where <interesting> means either a release branch or a channel, aka matches regex: "nixos-*"

Future: faster detection of above via webhook
