import {exec} from 'child_process';
import * as fs from 'fs';

import * as core from '@actions/core';
import * as github from '@actions/github';
import {downloadRelease} from '@terascope/fetch-github-release';

import {FileService} from './files';
import {annotationParams, parseAnnotation} from './annotations';

const isWin = process.platform === 'win32';
const file = 'missionreviewer.log';

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
  fs.readdirSync('.').forEach(file => {
    core.debug(file);
  });
  if (!isWin) {
    core.info('Setting permissions for missionreviewer');
    exec('chmod +x ./missionreviewer', (error, stdout, stderr) => {
      if (error) {
        core.setFailed(error.message);
      }
      if (stderr) {
        core.setFailed(stderr);
      }
      core.info(stdout);
    });
  }
  core.addPath(`${process.cwd()}`);

  let files: string[] = [];
  if (github.context.payload.pull_request) {
    files = await new FileService(
      core.getInput('GITHUB_TOKEN', {required: true})
    ).getFiles();
    core.debug(files.toString());
  }

  exec(
    `${process.cwd()}/${isWin ? 'missionreviewer.exe' : 'missionreviewer'}`,
    async (error, stdout, stderr) => {
      if (error) {
        console.error(`exec error: ${error}`);
        return;
      }
      console.log(`stdout: ${stdout}`);
      console.error(`stderr: ${stderr}`);

      if (!fs.existsSync(file)) {
        core.info('No annotations file found.');
        return;
      }
      const data = fs.readFileSync(file, 'utf8');
      const lines = data.split('\n');
      const annotations = lines
        .filter(line => line.length > 0)
        .map(parseAnnotation);
      core.info(`Found ${annotations.length} annotations.`);
      let approved = true;
      for (const annotation of annotations) {
        switch (annotation.level) {
          case 'error':
            core.error(annotation.message, annotationParams(annotation));
            if (
              annotation.path &&
              files.some(f => f.endsWith(annotation.path))
            ) {
              approved = false;
            }
            break;
          case 'warning':
            core.warning(annotation.message, annotationParams(annotation));
            break;
          default:
            core.notice(annotation.message, annotationParams(annotation));
            break;
        }
      }

      if (github.context.payload.pull_request) {
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
        if (approved) {
          options.body = 'Mission Reviewer: All checks passed!';
          options.event = 'APPROVE';
        } else {
          options.body = 'Mission Reviewer: Some checks failed.';
          options.event = 'REQUEST_CHANGES';
        }
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
        if (!approved || (approved && brodskycomments.length == 0)) {
          octo.rest.pulls.createReview(options);
        }
      }
    }
  );
}

run();
