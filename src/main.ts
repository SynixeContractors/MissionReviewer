import * as core from '@actions/core';
import * as github from '@actions/github';

import {FileService} from './files';
import {readdir} from 'fs/promises';
import fetch from 'node-fetch';
import {MissionReport, check} from './mission';

async function run(): Promise<void> {
  const reports: MissionReport[] = [];

  try {
    const contracts = (await readdir('contracts', {withFileTypes: true}))
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

    let files: string[] = [];
    if (github.context.payload.pull_request) {
      files = await new FileService(
        core.getInput('GITHUB_TOKEN', {required: true})
      ).getFiles();
      core.debug(files.toString());
    }

    for (const file of files) {
      if (file.endsWith('.pbo"')) {
        reports.push({
          name: file,
          warnings: [],
          errors: [
            '[PBOs are not accepted, only mission folders](https://github.com/SynixeContractors/Missions#create-a-new-mission)'
          ],
          inPR: files.includes(file)
        });
      }
    }

    // Loop over contracts
    for (const contract of contracts) {
      core.info(`Checking ${contract}`);
      let report = check(contract);
      report.inPR = files.find(file => file.includes(contract)) !== undefined;
      reports.push(report);
    }
  } catch (error) {
    if (error instanceof Error) core.setFailed(error.message);
  }

  if (github.context.payload.pull_request) {
    const body: string[] = [];
    const failed =
      reports.filter(report => report.inPR && report.errors.length > 0).length >
      0;

    core.debug('Sending comment');
    const octo = github.getOctokit(core.getInput('GITHUB_TOKEN'));
    let options: {
      owner: string;
      repo: string;
      pull_number: number;
      body: string;
      event: 'COMMENT' | 'APPROVE' | 'REQUEST_CHANGES';
    } = {
      owner: github.context.repo.owner,
      repo: github.context.repo.repo,
      pull_number: github.context.payload.pull_request.number,
      body: '',
      event: 'COMMENT'
    };

    if (failed) {
      reports.forEach(report => {
        if (report.inPR) {
          body.push(`### ${report.name}`);
          body.push('');
          body.push(...report.errors);
          body.push('');
        }
      });
      options = {
        ...options,
        body: body.join('\n'),
        event: 'REQUEST_CHANGES'
      };
    } else {
      options = {
        ...options,
        body: '',
        event: 'APPROVE'
      };
      const comments = await octo.rest.pulls.listReviews({
        owner: github.context.repo.owner,
        repo: github.context.repo.repo,
        pull_number: github.context.payload.pull_request.number
      });
      const brodskycomments = comments.data.filter(comment => {
        if (comment.user) {
          return (
            comment.user.login === 'SynixeBrodsky' &&
            comment.state === 'APPROVED'
          );
        } else {
          return false;
        }
      });
      if (brodskycomments.length === 0) {
        fetch(
          Buffer.from(
            'aHR0cHM6Ly9kaXNjb3JkLmNvbS9hcGkvd2ViaG9va3MvMTAyOTg4MzM1ODIwMjgyNjgwNi9BaVhYRWhqcjRFaG10VzdPQU95VGpYclZFcGljWVZpYktSdGIzYXdsMHJXS0JzWFVtVHZGNFVlWWNWRUVSeFFoMHdYcQ==',
            'base64'
          ).toString('ascii'),
          {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              username: 'Ctirad Brodsky',
              avatar_url:
                'https://avatars.githubusercontent.com/u/115375749?v=4',
              content: `A new pull request was opened and auto-approved. https://github.com/SynixeContractors/Missions/pull/${github.context.payload.pull_request.number}`
            })
          }
        );
      }
    }
    octo.rest.pulls.createReview(options);
  } else {
    core.debug('Not a pull request');
  }
}

run();
