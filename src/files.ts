import {getOctokit, context} from '@actions/github';
import * as core from '@actions/core';

export class FileService {
  private readonly token: string;

  constructor(token: string) {
    this.token = token;
  }

  async getFiles(): Promise<string[]> {
    let base: string;
    let head: string;

    switch (context.eventName) {
      case 'pull_request':
        base = context.payload.pull_request?.base?.sha;
        head = context.payload.pull_request?.head?.sha;
        break;
      case 'push':
        base = context.payload.before;
        head = context.payload.after;

        // special case for initial creation of branch
        if (+base === 0) {
          base = context.payload.base_ref
            ? context.payload.base_ref
            : context.payload.repository?.default_branch;
          core.info(`Switched Base to (${base}) for initial check-in.`);
        }
        break;
      default:
        throw new Error(
          'action must be used within a pull_request or push event'
        );
    }

    core.info(`Head SHA: ${head}`);
    core.info(`Base SHA: ${base}`);

    const response = await getOctokit(this.token).rest.repos.compareCommits({
      base,
      head,
      owner: context.repo.owner,
      repo: context.repo.repo
    });

    let files = response.data.files?.filter(x =>
      ['added', 'modified'].includes(x.status)
    );

    core.info(
      `Found (${files?.length}) ${files?.length === 1 ? 'File' : 'Files'}`
    );
    return files?.map(x => `"${x.filename}"`) || [];
  }
}
