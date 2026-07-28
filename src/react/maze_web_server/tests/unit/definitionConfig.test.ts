import { describe, it, expect } from 'vitest'
import {
  buildDefinitionConfig,
  parseDefinitionConfig,
  DEFINITION_DEFAULTS,
  type DefinitionFormState,
  type DefinitionConfig,
} from '../../src/utils/definitionConfig'

// A form with a value changed in each group, so the mapping is exercised end to
// end rather than through all-default values.
const sampleForm: DefinitionFormState = {
  ...DEFINITION_DEFAULTS,
  name: 'Tower',
  description: 'A tall one',
  generation: { ...DEFINITION_DEFAULTS.generation, rows: '12', cols: '10', minSolutionLength: '4', doorCount: '2', spareKeys: '3', treasureCount: '5' },
  scene: { ...DEFINITION_DEFAULTS.scene, skyType: 'day', wallType: 'wood', wallTint: true, wallMaterialVariation: true, perimeterWalls: false },
  objects: { ...DEFINITION_DEFAULTS.objects, doorStyle: 'portcullis', enemyType: 'ghost' },
  decor: { ...DEFINITION_DEFAULTS.decor, deadEndObjects: false },
  timerSeconds: '90',
  title: 'TOWER 3D',
  mode: 'Tower',
  minimapCellPx: '12',
  enemyMovePeriodMs: '1200',
  maxHp: '5',
  levels: { ...DEFINITION_DEFAULTS.levels, count: '3', taper: true, top: { skyType: 'sunset' } },
  visibility: 'public',
  rotation: 'daily',
  seed: 42,
}

const sampleMeta = { name: 'Tower', description: 'A tall one', visibility: 'public', rotation: 'daily' } as const

describe('buildDefinitionConfig', () => {
  it('parses numeric input strings into config numbers', () => {
    const { config } = buildDefinitionConfig(sampleForm)
    expect(config.rows).toBe(12)
    expect(config.cols).toBe(10)
    expect(config.minSolutionLength).toBe(4)
    expect(config.doorCount).toBe(2)
    expect(config.spareKeys).toBe(3)
    expect(config.treasureCount).toBe(5)
    expect(config.timerSeconds).toBe(90)
    expect(config.minimapCellPx).toBe(12)
    expect(config.enemyMovePeriodMs).toBe(1200)
    expect(config.maxHp).toBe(5)
    expect(config.levels.count).toBe(3)
    expect(config.seed).toBe(42)
  })

  it('folds scene + decor toggles into the nested landmarks object', () => {
    const { config } = buildDefinitionConfig(sampleForm)
    expect(config.landmarks).toEqual({
      wallTint: true, // scene
      wallMaterialVariation: true, // scene
      deadEndObjects: false, // decor
      wallDecorations: true, // decor default
      floorAccents: true, // decor default
    })
    // Top-level scene fields stay top-level.
    expect(config.skyType).toBe('day')
    expect(config.wallType).toBe('wood')
    expect(config.perimeterWalls).toBe(false)
    expect(config.doorStyle).toBe('portcullis')
    expect(config.enemyType).toBe('ghost')
  })

  it('builds the request with the definition-level fields + config', () => {
    const { config, request } = buildDefinitionConfig(sampleForm)
    expect(request.name).toBe('Tower')
    expect(request.description).toBe('A tall one')
    expect(request.visibility).toBe('public')
    expect(request.rotation).toBe('daily')
    expect(request.config).toBe(config as unknown as Record<string, unknown>)
  })

  it('stores an empty description as null', () => {
    const { request } = buildDefinitionConfig({ ...sampleForm, description: '   ' })
    expect(request.description).toBeNull()
  })

  it('carries the levels.top override through', () => {
    const { config } = buildDefinitionConfig(sampleForm)
    expect(config.levels.top).toEqual({ skyType: 'sunset' })
  })
})

describe('parseDefinitionConfig', () => {
  it('fills every group from defaults for an empty config', () => {
    const form = parseDefinitionConfig({}, { name: 'New', visibility: 'private', rotation: 'static' })
    expect(form.name).toBe('New')
    expect(form.description).toBe('')
    expect(form.generation.rows).toBe('8')
    expect(form.timerSeconds).toBe('120')
    expect(form.enemyMovePeriodMs).toBe('1500')
    expect(form.maxHp).toBe('3')
    expect(form.levels.count).toBe('1')
    expect(form.levels.finishType).toBe('ladder')
    expect(form.scene.skyType).toBe(DEFINITION_DEFAULTS.scene.skyType)
    expect(form.levels.top).toBeNull()
    expect(form.seed).toBe(0)
    expect(form.visibility).toBe('private')
    expect(form.rotation).toBe('static')
  })

  it('stringifies numeric config values', () => {
    const form = parseDefinitionConfig({ rows: 20, cols: 15, timerSeconds: 300 }, sampleMeta)
    expect(form.generation.rows).toBe('20')
    expect(form.generation.cols).toBe('15')
    expect(form.timerSeconds).toBe('300')
  })

  it('coerces an unrecognised enum value to its safe default', () => {
    const form = parseDefinitionConfig({ skyType: 'nonsense', wallType: 'wood' }, sampleMeta)
    expect(form.scene.skyType).toBe(DEFINITION_DEFAULTS.scene.skyType)
    expect(form.scene.wallType).toBe('wood')
  })

  it('keeps recognised level-enum values and coerces unrecognised ones', () => {
    const good = parseDefinitionConfig(
      { levels: { finishType: 'portal', difficultyChange: 'harder', alignment: 'random_base' } },
      sampleMeta,
    )
    expect(good.levels.finishType).toBe('portal')
    expect(good.levels.difficultyChange).toBe('harder')
    expect(good.levels.alignment).toBe('random_base')

    const bad = parseDefinitionConfig(
      { levels: { finishType: 'bogus', difficultyChange: 'nope', alignment: 'sideways' } },
      sampleMeta,
    )
    expect(bad.levels.finishType).toBe('ladder')
    expect(bad.levels.difficultyChange).toBe('easier')
    expect(bad.levels.alignment).toBe('edge')
  })

  it('drops an unrecognised top-level skyType override so it inherits the base', () => {
    const form = parseDefinitionConfig({ levels: { top: { skyType: 'bogus', perimeterWalls: true } } }, sampleMeta)
    expect(form.levels.top).toEqual({ perimeterWalls: true })
  })

  it('reads the nested landmarks back into scene + decor', () => {
    const form = parseDefinitionConfig(
      { landmarks: { wallTint: true, wallMaterialVariation: true, deadEndObjects: false, wallDecorations: false, floorAccents: true } },
      sampleMeta,
    )
    expect(form.scene.wallTint).toBe(true)
    expect(form.scene.wallMaterialVariation).toBe(true)
    expect(form.decor.deadEndObjects).toBe(false)
    expect(form.decor.wallDecorations).toBe(false)
    expect(form.decor.floorAccents).toBe(true)
  })

  it('takes name / description / visibility / rotation from meta, not the config', () => {
    const form = parseDefinitionConfig({ title: 'X' }, { name: 'Named', description: 'Desc', visibility: 'curated', rotation: 'daily' })
    expect(form.name).toBe('Named')
    expect(form.description).toBe('Desc')
    expect(form.visibility).toBe('curated')
    expect(form.rotation).toBe('daily')
    // title comes from the config, not meta.
    expect(form.title).toBe('X')
  })
})

describe('build → parse round-trip', () => {
  it('recovers the form state (via the built config + the same meta)', () => {
    const { config } = buildDefinitionConfig(sampleForm)
    const back = parseDefinitionConfig(config as unknown as Record<string, unknown>, sampleMeta)
    expect(back).toEqual(sampleForm)
  })

  it('round-trips the all-defaults form', () => {
    const { config } = buildDefinitionConfig(DEFINITION_DEFAULTS)
    const back = parseDefinitionConfig(config as unknown as Record<string, unknown>, {
      name: '',
      description: '',
      visibility: 'private',
      rotation: 'static',
    })
    expect(back).toEqual(DEFINITION_DEFAULTS)
  })
})

// Compile-time guard: the config type exposes no `difficulty` / `mazeJson` key.
describe('config shape', () => {
  it('omits difficulty and mazeJson', () => {
    const { config } = buildDefinitionConfig(sampleForm)
    expect('difficulty' in config).toBe(false)
    expect('mazeJson' in config).toBe(false)
    // Sanity: a couple of expected keys are present.
    const keys: (keyof DefinitionConfig)[] = ['rows', 'landmarks', 'levels']
    for (const k of keys) expect(k in config).toBe(true)
  })
})
