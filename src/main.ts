import * as core from '@actions/core'

import {existsSync, readFileSync} from 'fs'
import {join} from 'path'
import {readdir} from 'fs/promises'

const regex_desc_name = /OnLoadName = "(.+?)";/gm
const regex_desc_summary = /OnLoadMission = "(.+?)";/gm
const regex_desc_author = /author = "(.+?)";/gm

async function run(): Promise<void> {
  try {
    const contracts = (await readdir('contracts', {withFileTypes: true}))
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name)
    // Loop over contracts
    for (const contract of contracts) {
      core.info(`Checking ${contract}`)
      const description_path = join(
        'contracts',
        contract,
        'edit_me/description.ext'
      )
      if (!existsSync(description_path)) {
        core.info('- Not using template')
        return
      }

      // Check Description
      const description = readFileSync(description_path, 'utf8')
      // Description - Check Name
      if (regex_desc_name.exec(description) === null) {
        core.error(`- Description: Name not set (OnLoadName)`)
      }
      // Description - Check Summary
      if (regex_desc_summary.exec(description) === null) {
        core.error(`- Description: Summary not set (OnLoadMission)`)
      }
      // Description - Check Author
      if (regex_desc_author.exec(description) === null) {
        core.error(`- Description: Author not set (author)`)
      }

      // Check mission.sqm
      const mission_path = join('contracts', contract, 'mission.sqm')
      if (!existsSync(mission_path)) {
        core.error(`- mission.sqm not found`)
        return
      }
      const mission = readFileSync(mission_path, 'utf8')

      // Mission - Spectator Screen
      if (!mission.includes('type="synixe_spectator_screen"')) {
        core.error(`- mission.sqm: Spectator Screen not found`)
      }

      // Mission - Check Shop
      if (
        !mission.includes(
          'property="persistent_gear_shop_arsenal_attribute_shop"'
        )
      ) {
        core.error(`- mission.sqm: Shop not found`)
      }

      // Mission - Check Respawn
      if (!mission.includes('name="respawn"')) {
        core.error(`- mission.sqm: Respawn not found`)
      }

      // Mission - Has Contractors
      if (!mission.includes('description="Contractor"')) {
        core.error(`- mission.sqm: No "Contractor" units found`)
      }

      // Mission - Uses Synixe Unit Class
      if (!mission.includes('type="synixe_contractors_Unit_I_Contractor"')) {
        core.error(
          `- mission.sqm: No "synixe_contractors_Unit_I_Contractor" units found`
        )
      }

      // Mission - Playable Units
      if (!mission.includes('isPlayable=1')) {
        core.error(`- mission.sqm: No playable units found`)
      }
    }
  } catch (error) {
    if (error instanceof Error) core.setFailed(error.message)
  }
}

run()
