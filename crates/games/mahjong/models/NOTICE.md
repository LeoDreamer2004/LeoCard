# kdens3

The optional kdens3 checkpoints are published by Dannibal / SuuTTT under MIT,
as declared in the pinned model card:
https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/blob/6964d80b9f8a6670dda44f14b664c61112f61009/README.md

Inference architecture and feature contract:
https://huggingface.co/Dannibal/ijcai-mahjong-ckpts-2026/tree/6964d80b9f8a6670dda44f14b664c61112f61009/champion

The Rust implementation follows the published 38-plane input, 235-action vocabulary,
fused residual network and mean-softmax ensemble. It preserves the feature agent's
own-hand refresh points. Weights are downloaded separately and never embedded.

The GitHub code URL advertised by that model card returned HTTP 404 when checked
on 2026-11-25. The application links to the accessible Hugging Face model page;
its champion directory also contains the published inference and feature code.
