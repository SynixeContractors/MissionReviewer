import * as fs from 'fs';

import * as core from '@actions/core';
import * as github from '@actions/github';

/** Written by the reviewer: maps used by missions that are missing from maps.txt */
const MISSING_MAPS = 'missionreviewer.maps';
const MAPS_TXT = 'maps.txt';
const BRANCH = 'missionreviewer/maps';

function readMissing(): string[] {
  if (!fs.existsSync(MISSING_MAPS)) {
    return [];
  }
  return fs
    .readFileSync(MISSING_MAPS, 'utf8')
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0);
}

function listedMaps(content: string): string[] {
  return content
    .split('\n')
    .map(line => line.trim().toLowerCase())
    .filter(line => line.length > 0 && !line.startsWith('#'));
}

/**
 * Open (or update) a pull request adding maps that merged missions use to maps.txt.
 * Only runs for pushes to the default branch, so maps from unmerged pull requests are not added.
 */
export async function proposeMissingMaps(token: string): Promise<void> {
  const missing = readMissing();
  if (missing.length === 0) {
    return;
  }
  const context = github.context;
  const defaultBranch: string | undefined =
    context.payload.repository?.default_branch;
  if (
    context.eventName !== 'push' ||
    !defaultBranch ||
    context.ref !== `refs/heads/${defaultBranch}`
  ) {
    core.info(
      `Maps missing from ${MAPS_TXT}: ${missing.join(', ')}. A pull request is opened after merging.`
    );
    return;
  }

  const octokit = github.getOctokit(token);
  const {owner, repo} = context.repo;

  const open = await octokit.rest.pulls.list({
    owner,
    repo,
    state: 'open',
    head: `${owner}:${BRANCH}`
  });
  if (open.data.length > 0) {
    // Leave the open pull request alone if it already adds every missing map
    const onBranch = await octokit.rest.repos.getContent({
      owner,
      repo,
      path: MAPS_TXT,
      ref: BRANCH
    });
    if (!Array.isArray(onBranch.data) && onBranch.data.type === 'file') {
      const listed = listedMaps(
        Buffer.from(onBranch.data.content, 'base64').toString('utf8')
      );
      if (missing.every(map => listed.includes(map.toLowerCase()))) {
        core.info(
          `Pull request #${open.data[0].number} already adds ${missing.join(', ')}`
        );
        return;
      }
    }
  }

  const current = await octokit.rest.repos.getContent({
    owner,
    repo,
    path: MAPS_TXT,
    ref: context.sha
  });
  if (Array.isArray(current.data) || current.data.type !== 'file') {
    core.warning(`${MAPS_TXT} is not a file`);
    return;
  }
  const content =
    Buffer.from(current.data.content, 'base64')
      .toString('utf8')
      .replace(/\s*$/, '\n') + missing.map(map => `${map}\n`).join('');

  // Start the branch again from the pushed commit, so the pull request only changes maps.txt
  try {
    await octokit.rest.git.updateRef({
      owner,
      repo,
      ref: `heads/${BRANCH}`,
      sha: context.sha,
      force: true
    });
  } catch {
    await octokit.rest.git.createRef({
      owner,
      repo,
      ref: `refs/heads/${BRANCH}`,
      sha: context.sha
    });
  }
  await octokit.rest.repos.createOrUpdateFileContents({
    owner,
    repo,
    path: MAPS_TXT,
    branch: BRANCH,
    sha: current.data.sha,
    message: `Add ${missing.join(', ')} to ${MAPS_TXT}`,
    content: Buffer.from(content, 'utf8').toString('base64')
  });

  const body = [
    `Missions on these maps were merged, but the maps are not in \`${MAPS_TXT}\`:`,
    '',
    ...missing.map(map => `* \`${map}\``),
    '',
    'They are added at the end of the file, move them to the right section if needed.',
    '',
    'Opened by the mission reviewer.'
  ].join('\n');
  if (open.data.length > 0) {
    await octokit.rest.pulls.update({
      owner,
      repo,
      pull_number: open.data[0].number,
      title: `Add missing maps to ${MAPS_TXT}`,
      body
    });
    core.info(`Updated pull request #${open.data[0].number}`);
  } else {
    const created = await octokit.rest.pulls.create({
      owner,
      repo,
      head: BRANCH,
      base: defaultBranch,
      title: `Add missing maps to ${MAPS_TXT}`,
      body
    });
    core.info(`Opened pull request #${created.data.number}`);
  }
}
