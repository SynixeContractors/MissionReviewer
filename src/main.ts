import * as core from '@actions/core';
import * as github from '@actions/github';

import {existsSync, readFileSync} from 'fs';
import {join} from 'path';
import {readdir} from 'fs/promises';
import {FileService} from './files';

const regex_desc_name = /OnLoadName = "(.+?)";/gm;
const regex_desc_summary = /OnLoadMission = "(.+?)";/gm;
const regex_desc_author = /author = "(.+?)";/gm;

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
      console.log(files);
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
        core.info('- Not using template');
        return;
      }

      // Check Description
      const description = readFileSync(description_path, 'utf8');
      // Description - Check Name
      if (regex_desc_name.exec(description) === null) {
        core.error(`${contract} - Description: Name not set (OnLoadName)`);
        error && messages.push(`description.ext: Name not set (OnLoadName)`);
      }
      // Description - Check Summary
      if (regex_desc_summary.exec(description) === null) {
        core.error(
          `${contract} - Description: Summary not set (OnLoadMission)`
        );
        error &&
          messages.push(`description.ext: Summary not set (OnLoadMission)`);
      }
      // Description - Check Author
      if (regex_desc_author.exec(description) === null) {
        core.error(`${contract} - Description: Author not set (author)`);
      }

      // Check mission.sqm
      const mission_path = join('contracts', contract, 'mission.sqm');
      if (!existsSync(mission_path)) {
        core.error(`${contract} - mission.sqm not found`);
        return;
      }
      const mission = readFileSync(mission_path, 'utf8');

      // Mission - Spectator Screen
      if (!mission.includes('type="synixe_spectator_screen"')) {
        core.error(`${contract} - mission.sqm: Spectator Screen not found`);
        error && messages.push(`Spectator Screen not found`);
      }

      // Mission - Check Shop
      if (
        !mission.includes(
          'property="persistent_gear_shop_arsenal_attribute_shop"'
        )
      ) {
        core.error(`${contract} - mission.sqm: Shop not found`);
        error && messages.push(`Shop not found`);
      }

      // Mission - Check Respawn
      if (!mission.includes('name="respawn"')) {
        core.error(`${contract} - mission.sqm: Respawn not found`);
        error && messages.push(`Respawn not found`);
      }

      // Mission - Has Contractors
      if (!mission.includes('description="Contractor"')) {
        core.error(`${contract} - mission.sqm: No "Contractor" units found`);
        error && messages.push(`No "Contractor" units found`);
      }

      // Mission - Uses Synixe Unit Class
      if (!mission.includes('type="synixe_contractors_Unit_I_Contractor"')) {
        core.error(
          `${contract} - mission.sqm: No "synixe_contractors_Unit_I_Contractor" units found`
        );
        error &&
          messages.push(
            `No "synixe_contractors_Unit_I_Contractor" units found`
          );
      }

      // Mission - Playable Units
      if (!mission.includes('isPlayable=1')) {
        core.error(`${contract} - mission.sqm: No playable units found`);
        error && messages.push(`No playable units found`);
      }

      error && body.push(messages);
    }
  } catch (error) {
    if (error instanceof Error) core.setFailed(error.message);
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
    if (body.length == 0) {
      options = {
        ...options,
        body: '',
        event: 'APPROVE'
      };
    } else {
      options = {
        ...options,
        body: body.map(m => m.join('\n')).join('\n'),
        event: 'REQUEST_CHANGES'
      };
    }
    octo.rest.pulls.createReview(options);
  }
}

run();
