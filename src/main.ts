import * as core from '@actions/core';
import * as github from '@actions/github';

import {existsSync, readFileSync} from 'fs';
import {FileService} from './files';
import {join} from 'path';
import {readdir} from 'fs/promises';
import fetch from 'node-fetch';

const regex_desc_name = /^OnLoadName = "(.+?)";$/m;
const regex_desc_summary = /^OnLoadMission = "(.+?)";$/m;
const regex_desc_author = /^author = "(.+?)";$/m;

async function run(): Promise<void> {
  const body = [];

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
        body.push([
          '**' + file + '**',
          '[PBOs are not accepted, only mission folders](https://github.com/SynixeContractors/Missions#create-a-new-mission)'
        ]);
      }
    }

    // Loop over contracts
    for (const contract of contracts) {
      const messages: string[] = ['**' + contract + '**'];
      let error = files.find(file => file.includes(contract));
      core.info(`Checking ${contract}`);
      const description_path = join(
        'contracts',
        contract,
        'edit_me/description.ext'
      );
      if (!existsSync(description_path)) {
        core.info(`${contract} - Not using template`);
        error &&
          messages.push(
            '[Not using template](https://github.com/SynixeContractors/MissionTemplate)'
          );
      }
      if (existsSync(description_path)) {
        // Check Description
        const description = readFileSync(description_path, 'utf8');
        // Description - Check Name
        if (regex_desc_name.exec(description) === null) {
          core.error(`${contract} - Description: Name not set (OnLoadName)`);
          error &&
            messages.push(
              `[description.ext: Name not set (OnLoadName)](https://github.com/SynixeContractors/MissionTemplate#mission-details)`
            );
        }
        // Description - Check Summary
        if (regex_desc_summary.exec(description) === null) {
          core.error(
            `${contract} - Description: Summary not set (OnLoadMission)`
          );
          error &&
            messages.push(
              `[description.ext: Summary not set (OnLoadMission)](https://github.com/SynixeContractors/MissionTemplate#mission-details)`
            );
        }
        // Description - Check Author
        if (regex_desc_author.exec(description) === null) {
          core.error(`${contract} - Description: Author not set (author)`);
          error &&
            messages.push(
              `[description.ext: Author not set (author)](https://github.com/SynixeContractors/MissionTemplate#mission-details)`
            );
        }
      }

      // Check mission.sqm
      const mission_path = join('contracts', contract, 'mission.sqm');
      if (!existsSync(mission_path)) {
        core.error(`${contract} - mission.sqm not found`);
      }
      if (existsSync(mission_path)) {
        const mission = readFileSync(mission_path, 'utf8');
        if (mission.startsWith('version')) {
          // Mission - Spectator Screen
          if (!mission.includes('type="synixe_spectator_screen"')) {
            core.error(`${contract} - mission.sqm: Spectator Screen not found`);
            error &&
              messages.push(
                `[Spectator Screen not found](https://github.com/SynixeContractors/MissionTemplate#setup-base)`
              );
          }

          // Mission - Check Respawn
          if (!mission.includes('name="respawn"')) {
            core.error(`${contract} - mission.sqm: Respawn not found`);
            error &&
              messages.push(
                `[Respawn not found](https://github.com/SynixeContractors/MissionTemplate#setup-base)`
              );
          }

          // Mission - Check Shop
          if (
            !mission.includes('property="crate_client_gear_attribute_shop"')
          ) {
            core.error(`${contract} - mission.sqm: Shop not found`);
            error &&
              messages.push(
                `[Shop not found](https://github.com/SynixeContractors/MissionTemplate#setup-shops)`
              );
          }

          // Mission - Has Contractors
          if (!mission.includes('description="Contractor"')) {
            core.error(
              `${contract} - mission.sqm: No "Contractor" units found`
            );
            error &&
              messages.push(
                `[No "Contractor" units found](https://github.com/SynixeContractors/MissionTemplate#setup-the-players)`
              );
          }

          // Mission - Uses Synixe Unit Class
          if (
            !mission.includes('type="synixe_contractors_Unit_I_Contractor"')
          ) {
            core.error(
              `${contract} - mission.sqm: No "synixe_contractors_Unit_I_Contractor" units found`
            );
            error &&
              messages.push(
                `[No "synixe_contractors_Unit_I_Contractor" units found](https://github.com/SynixeContractors/MissionTemplate#setup-the-players)`
              );
          }

          // Mission - Playable Units
          if (!mission.includes('isPlayable=1')) {
            core.error(`${contract} - mission.sqm: No playable units found`);
            error &&
              messages.push(
                `[No playable units found](https://github.com/SynixeContractors/MissionTemplate#setup-the-players)`
              );
          }
        } else {
          core.error(`${contract} - mission.sqm: Binarized`);
          error &&
            messages.push(
              '[mission.sqm: Binarized](https://github.com/SynixeContractors/Missions#create-a-new-mission)'
            );
        }
      }

      // Check briefing.sqf
      const briefing_path = join(
        'contracts',
        contract,
        'edit_me',
        'briefing.sqf'
      );
      if (!existsSync(briefing_path)) {
        core.error(`${contract} - briefing.sqf not found`);
      }
      if (existsSync(briefing_path)) {
        const briefing = readFileSync(briefing_path, 'utf8');
        if (briefing.includes('INSERT NAME OF EMPLOYER HERE')) {
          core.error(`${contract} - briefing.sqf: Employer not set`);
          error && messages.push(`briefing.sqf: Employer not set`);
        }
        if (briefing.includes('INSERT ENEMIES HERE')) {
          core.error(`${contract} - briefing.sqf: Situation not set`);
          error && messages.push(`briefing.sqf: Situation not set`);
        }
        if (briefing.includes('YOU CAN WRITE YOUR MISSION DESCRIPTION HERE')) {
          core.error(`${contract} - briefing.sqf: Mission not set`);
          error && messages.push(`briefing.sqf: Mission not set`);
        }
      }

      error && body.push(messages);
    }
  } catch (error) {
    if (error instanceof Error) core.setFailed(error.message);
  }

  if (github.context.payload.pull_request) {
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
    if (body.every(messages => messages.length === 1)) {
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
    } else {
      options = {
        ...options,
        body: body.map(m => m.join('\n')).join('\n'),
        event: 'REQUEST_CHANGES'
      };
    }
    octo.rest.pulls.createReview(options);
  } else {
    core.debug('Not a pull request');
  }
}

run();
