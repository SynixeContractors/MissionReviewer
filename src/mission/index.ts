import * as core from '@actions/core';

import {existsSync, readFileSync} from 'fs';
import {join} from 'path';
import {check2} from './versions/2';
import {check3} from './versions/3';

export type MissionReport = {
  name: string;
  warnings: string[];
  errors: string[];
  inPR: boolean;
};

export function check(name: string): MissionReport {
  const description_path = join(
    'contracts',
    name,
    'do_not_edit/description.ext'
  );
  if (!existsSync(description_path)) {
    core.info(`${name} - Not using template`);
    return {
      name,
      warnings: [],
      errors: [
        '[Not using template](https://github.com/SynixeContractors/MissionTemplate)'
      ],
      inPR: false
    };
  }

  // Get Version
  const description = readFileSync(description_path, 'utf8');
  const version_exec = /^synixe_template = (\d+);/m.exec(description);
  let version = 2;
  if (version_exec !== null) {
    version = parseInt(version_exec[1]);
  }
  console.log(`${name} - Using template: v${version}`);

  switch (version) {
    case 2:
      let report = check2(name);
      report.warnings.push('`Using old template: v2`');
      return report;
    case 3:
      return check3(name);
    default:
      return {
        name,
        warnings: [],
        errors: [`Unknown version: ${version}`],
        inPR: false
      };
  }
}
