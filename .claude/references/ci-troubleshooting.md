# CI Troubleshooting Contract

`runner_id=0` with `steps=[]` means jobs did not execute. It is not a code-quality result.

Required report fields:

- workflow run ID and URL
- job count and each job conclusion
- `runner_id` and `steps`
- whether checkout happened
- API error/status if dispatch failed

Never call CI green when no runner executed jobs. Track quota, billing, repository Actions policy, organization policy, and token permissions separately.
