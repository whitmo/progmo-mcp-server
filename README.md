# [p]rog[mo]

_program more_

An agent for handling out of band common coding tasks

- knowledge management
- documentation driven development
- code review
- test writing and running and gardening


## Knowledge Management

p-mo works with a vector datastore to provide specific context to Cline and other mcp clients. It also also for the basic crud operations of tokenizing text sources, uploading them to the vector store and deleting them when no longer needed.

Supported
- [Qdurant](https://github.com/qdrant/qdrant): containerized locally run vector store.

### Research
Graph RAG
- https://www.semanticpartners.com/post/a-triple-store-rag-retriever
- https://github.com/indradb/indradb

## DDD manager

Starting with the high level "why", manages our critical path and the narrative of our build. Records features and their user stories, technical specification, decisions and everything needed for operating our system.  Uses the "external brain" format for knowledge management: projects for planning active efforts (w/ completion horizons), resources for active reference and policy material, archive for archived resources and projects.


## Code Review

Using a `.codereview` file or the `.clinefile` as a guide (plus any initial prompting), cuts a review branch and iterates over the code, adds tests, runs tests, makes adjustments. Can be run in a constant mode which periodically pulls in latest commits and keeps a change log of review comments and change commits

## Test manager

In some ways, more or less the same as "Code Review", but focussed only on running and fixing tests, improving coverage, and changes to the code to improve testing isolation, speed, layering, etc.  Can provide "last run" data for a repo's tests.



# Implementation

Writing this as a rust server distributed as a binary or container.
