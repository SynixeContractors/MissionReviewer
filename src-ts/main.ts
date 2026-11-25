import {execFile} from 'child_process';
import * as fs from 'fs';
import {promisify} from 'util';

import * as core from '@actions/core';
import * as github from '@actions/github';
import {downloadRelease} from '@terascope/fetch-github-release';

import {FileService} from './files';
import {annotationParams, parseAnnotation} from './annotations';
import {proposeMissingMaps} from './maps';

const isWin = process.platform === 'win32';
const file = 'missionreviewer.log';
const execFileAsync = promisify(execFile);

async function run(): Promise<void> {
  await downloadRelease(
    'SynixeContractors',
    'MissionReviewer',
    'missionreviewer',
    release => {
      return release.prerelease === false;
    },
    asset => {
      return isWin
        ? asset.name === 'missionreviewer.exe'
        : asset.name === 'missionreviewer';
    },
    false,
    false
  );
  const binary = `${process.cwd()}/missionreviewer/${isWin ? 'missionreviewer.exe' : 'missionreviewer'}`;
  if (!isWin) {
    fs.chmodSync(binary, 0o755);
    core.info('missionreviewer is now executable');
  }

  let files: string[] = [];
  if (github.context.payload.pull_request) {
    files = await new FileService(
      core.getInput('GITHUB_TOKEN', {required: true})
    ).getFiles();
    core.debug(files.toString());
  }
  const isChanged = (path: string): boolean =>
    path.length > 0 && files.some(f => f.includes(path));

  // Give the filesystem a moment after downloading, executing a freshly written binary can fail with ETXTBSY
  await new Promise(resolve => setTimeout(resolve, 3000));

  // Only a failure to run the reviewer fails the job, review findings never do
  for (const output of [file, 'missionreviewer.maps']) {
    if (fs.existsSync(output)) {
      fs.unlinkSync(output);
    }
  }
  try {
    const {stdout, stderr} = await execFileAsync(binary, [], {
      maxBuffer: 64 * 1024 * 1024
    });
    core.info(stdout);
    if (stderr) {
      core.warning(stderr);
    }
  } catch (error) {
    core.setFailed(`missionreviewer failed to run: ${error}`);
    return;
  }

  if (!fs.existsSync(file)) {
    core.setFailed('missionreviewer did not write an annotations file.');
    return;
  }
  const annotations = fs
    .readFileSync(file, 'utf8')
    .split('\n')
    .filter(line => line.length > 0)
    .map(parseAnnotation);
  core.info(`Found ${annotations.length} annotations.`);

  // GitHub only shows a limited number of annotations, so show the ones for changed files first
  annotations.sort(
    (a, b) => Number(isChanged(b.path)) - Number(isChanged(a.path))
  );

  let approved = true;
  const messages: {[key: string]: string[]} = {};
  for (const annotation of annotations) {
    const changed = isChanged(annotation.path);
    switch (annotation.level) {
      case 'error':
        core.error(annotation.message, annotationParams(annotation));
        if (changed) {
          approved = false;
          messages[annotation.path] = [
            ...(messages[annotation.path] || []),
            annotation.message
          ];
        }
        break;
      case 'warning':
        core.warning(annotation.message, annotationParams(annotation));
        if (changed) {
          messages[annotation.path] = [
            ...(messages[annotation.path] || []),
            annotation.message
          ];
        }
        break;
      default:
        core.notice(annotation.message, annotationParams(annotation));
        break;
    }
  }

  try {
    await proposeMissingMaps(core.getInput('GITHUB_TOKEN'));
  } catch (error) {
    core.warning(`Failed to open a pull request for missing maps: ${error}`);
  }

  const pullRequest = github.context.payload.pull_request;
  if (!pullRequest) {
    return;
  }

  let body = '';
  if (!approved) {
    body = 'Found issues with the following files:\n';
    for (const path in messages) {
      body += `* ${path}\n`;
      for (const message of new Set(messages[path])) {
        body += `  * ${message.replace(/\n/g, '\n    ')}\n`;
      }
    }
  }
  try {
    const octokit = github.getOctokit(core.getInput('GITHUB_TOKEN'));
    await octokit.rest.pulls.createReview({
      owner: github.context.repo.owner,
      repo: github.context.repo.repo,
      pull_number: pullRequest.number,
      body,
      event: approved ? 'APPROVE' : 'REQUEST_CHANGES'
    });
  } catch (error) {
    core.warning(`Failed to submit the review: ${error}`);
  }
}

run().catch(error => core.setFailed(`${error}`));
