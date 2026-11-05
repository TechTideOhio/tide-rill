// Synthetic-only pilot. Loads a local patched repository, never sends corpus or queries off localhost.
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';

const baseUrl = process.env.PILOT_WEAVIATE_URL;
if (!/^http:\/\/127\.0\.0\.1:\d+$/.test(baseUrl ?? '')) throw Error('Set PILOT_WEAVIATE_URL to the dedicated loopback service');
const sourcePath = process.env.PILOT_REPOSITORY_SOURCE;
if (!sourcePath) throw Error('Set PILOT_REPOSITORY_SOURCE to the patched repository source');
const outputPath = process.env.PILOT_OUTPUT;
if (!outputPath) throw Error('Set PILOT_OUTPUT to a new output path');
const sha = 'ee82ad39f8e322d5ca2c4cf4a474c993a646eb95';
const base = `https://raw.githubusercontent.com/Alexi5000/TechTideAI2/${sha}/backend/src`;
const get = async (url) => { const r = await fetch(url); if (!r.ok) throw Error(`Source fetch failed: ${r.status}`); return r.text(); };
const [repositorySource, serviceSource, corpusText, draftText] = await Promise.all([
  readFile(sourcePath, 'utf8'), get(`${base}/services/weaviate.ts`),
  readFile(new URL('./corpus.json', import.meta.url), 'utf8'), readFile(new URL('./draft-judgments.json', import.meta.url), 'utf8'),
]);
const data = (s) => `data:text/javascript;base64,${Buffer.from(s).toString('base64')}`;
const domain = data('export class VectorStoreUnavailableError extends Error {}');
const env = data(`export const env = ${JSON.stringify({ WEAVIATE_URL: baseUrl })};`);
const service = serviceSource.replace('"../config/env.js"', JSON.stringify(env)).replace('"../domain/index.js"', JSON.stringify(domain));
const serviceUrl = data(stripTypeScriptTypes(service));
const repository = repositorySource.replace('"../services/weaviate.js"', JSON.stringify(serviceUrl)).replace('"../domain/index.js"', JSON.stringify(domain));
const { createKnowledgeVectorRepository } = await import(data(stripTypeScriptTypes(repository)));
const corpus = JSON.parse(corpusText);
const draft = JSON.parse(draftText);
// A deterministic toy lexical feature hash, NOT a semantic embedding model.
const vector = (text) => {
  const result = Array(512).fill(0);
  for (const token of text.toLowerCase().match(/[a-z0-9]+/g) ?? []) {
    const n = createHash('sha256').update(token).digest().readUInt32BE(0) % result.length;
    result[n] += 1;
  }
  const length = Math.hypot(...result);
  if (!length) throw Error('empty feature vector');
  return result.map(x => x / length);
};
const repo = createKnowledgeVectorRepository();
await repo.ensureSchema();
await repo.upsertChunks(corpus.chunks.map(c => ({ id: c.id, orgId: 'synthetic-pilot', documentId: c.document_id, chunkIndex: c.chunk_index, title: c.title, source: 'synthetic', collection: null, content: c.content, vector: vector(`${c.title} ${c.content}`) })));
// The repository checks the batch HTTP status, not individual errors; verify actual stored IDs.
for (const chunk of corpus.chunks) {
  const r = await fetch(`${baseUrl}/v1/objects/KnowledgeChunkV2/${chunk.id}`);
  if (!r.ok) throw Error('A chunk was not ingested');
}
const results = [];
for (const query of draft.queries) {
  const found = await repo.search({ orgId: 'synthetic-pilot', vector: vector(query.text), limit: 6 });
  if (found.some(x => !corpus.chunks.some(c => c.id === x.chunkId)) || new Set(found.map(x => x.chunkId)).size !== found.length) throw Error('Invalid retrieved chunk IDs');
  results.push({ id: query.id, status: 'success', ranking: found.map(x => ({ id: x.chunkId, distance: x.distance })) });
}
const digest = (text) => createHash('sha256').update(text).digest('hex');
await writeFile(outputPath, JSON.stringify({ version: 1, repo_base_commit: sha, repository_source: 'local patched working tree', repository_source_sha256: digest(repositorySource), service_source_sha256: digest(serviceSource), corpus_sha256: digest(corpusText), judgments_sha256: digest(draftText), embedding: 'local lexical SHA-256 feature hash, 512 dimensions, L2 normalized', limit: 6, results }, null, 2) + '\n', { flag: 'wx' });
console.log(`Captured ${results.length} real Weaviate searches using the locally patched TechTideAI2 repository.`);
