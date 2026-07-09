import { useState } from 'react'
import { StepModalShell } from './StepModalShell'
import { MazeGenerationFields, type MazeGenerationFieldsValue } from './MazeGenerationFields'
import { ObjectGroupsFields } from './ObjectGroupsFields'
import { GameSceneFields } from './GameSceneFields'
import type {
  SceneFieldsValue,
  ObjectsFieldsValue,
  DecorFieldsValue,
} from './GameSettingsFields'
import { LevelProgressionFields } from './LevelProgressionFields'
import { LevelSettingsFields } from './LevelSettingsFields'
import { FieldGroup } from './FieldGroup'
import { AdvancedFields, type AdvancedFieldsValue } from './AdvancedFields'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'
import { modalTabPanelProps, type WizardStep } from '../utils/modalTabs'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { validateMazeGenerationFields } from '../utils/validation'
import { MAX_LEVEL_COUNT, FINISH_TYPES, type FinishType } from '../utils/gameDefinitions'
import { titleCaseWire } from '../utils/cellEntityStyles'
import { buildDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import type { GameDefinitionRequest } from '../types/api'

// The game-definition editor: the definition's own General details (name,
// description, level count + time limit) plus the scene (incl. decor) / layout /
// objects / advanced field-groups, hosted in the dual-mode step shell — a wizard
// for creating a definition, tabs for editing one. The multi-level settings are
// distributed across those tabs (revealed when the count is above 1) rather than
// a dedicated Levels tab. It owns the working form state and hands the caller a
// finished `GameDefinitionRequest`, so create and edit differ only in `mode`, the
// seed state and what the caller does with the request.

const STEPS = [
  { id: 'general', label: 'General' },
  { id: 'scene', label: 'Scene' },
  { id: 'layout', label: 'Layout' },
  { id: 'objects', label: 'Objects' },
  { id: 'advanced', label: 'Advanced' },
] as const satisfies readonly WizardStep[]

type EditorStep = (typeof STEPS)[number]['id']

const ID_PREFIX = 'gamedef'

interface GameDefinitionEditorProps {
  mode: 'tabs' | 'wizard'
  /** Seed state — the defaults for a new definition, or a parsed stored one. */
  initialForm: DefinitionFormState
  title: string
  /** Commit-button label; defaults to the shell's Finish (wizard) / Save (tabs). */
  commitLabel?: string
  onSubmit: (request: GameDefinitionRequest) => void
  onCancel: () => void
}

export function GameDefinitionEditor({
  mode,
  initialForm,
  title,
  commitLabel,
  onSubmit,
  onCancel,
}: GameDefinitionEditorProps) {
  const { max_maze_cells } = useAppFeatures()
  const [activeStep, setActiveStep] = useState<EditorStep>('general')
  const [form, setForm] = useState<DefinitionFormState>(initialForm)

  // Whether the game stacks multiple levels (count ≤ 1, incl. blank/invalid, is
  // single-level). The multi-level-only controls — the Layout Levels group, the
  // Scene final-level overrides, the Objects Finish Cell, and the Advanced Levels
  // group — are revealed by this rather than a dedicated tab.
  const isMultiLevel = parseInt(form.levels.count, 10) > 1

  const patchGeneration = (patch: Partial<MazeGenerationFieldsValue>) =>
    setForm(f => ({ ...f, generation: { ...f.generation, ...patch } }))
  const patchScene = (patch: Partial<SceneFieldsValue>) =>
    setForm(f => ({ ...f, scene: { ...f.scene, ...patch } }))
  const patchObjects = (patch: Partial<ObjectsFieldsValue>) =>
    setForm(f => ({ ...f, objects: { ...f.objects, ...patch } }))
  const patchDecor = (patch: Partial<DecorFieldsValue>) =>
    setForm(f => ({ ...f, decor: { ...f.decor, ...patch } }))
  const patchLevels = (patch: Partial<DefinitionLevelsFormValue>) =>
    setForm(f => ({ ...f, levels: { ...f.levels, ...patch } }))
  // The advanced fields are all top-level on the form state, so a patch merges
  // straight in.
  const patchAdvanced = (patch: Partial<AdvancedFieldsValue>) => setForm(f => ({ ...f, ...patch }))

  // The commit gate: a name and generation settings the generator would accept.
  // Every other field is defaulted, which is what makes the wizard's early
  // Finish (from any step) safe. The generation error also shows in the pinned
  // footer, so it stays visible while the user is on another step.
  const generationError = validateMazeGenerationFields(form.generation, max_maze_cells, 'game')
  const canCommit = form.name.trim() !== '' && generationError === null

  function handleCommit() {
    const name = form.name.trim()
    // The in-game splash title and status-bar label default to the game's name
    // when left blank, so a definition always announces itself as something.
    onSubmit(
      buildDefinitionConfig({
        ...form,
        name,
        title: form.title.trim() === '' ? name : form.title,
        mode: form.mode.trim() === '' ? name : form.mode,
      }).request,
    )
  }

  return (
    <StepModalShell
      mode={mode}
      title={title}
      steps={STEPS}
      activeStep={activeStep}
      onStepChange={setActiveStep}
      idPrefix={ID_PREFIX}
      ariaLabel="Game definition steps"
      onCancel={onCancel}
      onCommit={handleCommit}
      canCommit={canCommit}
      commitLabel={commitLabel}
      footerNote={generationError && <p role="alert" className="error-msg">{generationError}</p>}
    >
      <div {...modalTabPanelProps(ID_PREFIX, 'general', activeStep)}>
        <FieldGroup title="Details" id="details">
          <label className="modal-stacked-input">
            Name
            <input
              type="text"
              className="input"
              value={form.name}
              onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
            />
          </label>
          <label className="modal-stacked-input">
            Description
            <textarea
              className="input"
              rows={3}
              value={form.description}
              onChange={e => setForm(f => ({ ...f, description: e.target.value }))}
            />
          </label>
        </FieldGroup>

        {/* Single-field groups: the group heading is the field's label, so the
            lone input carries an aria-label for its accessible name. */}
        <FieldGroup title="Number of Levels" id="number-of-levels">
          <input
            type="number"
            className="input"
            aria-label="Number of Levels"
            value={form.levels.count}
            min={1}
            max={MAX_LEVEL_COUNT}
            onChange={e => patchLevels({ count: e.target.value })}
          />
        </FieldGroup>

        <FieldGroup title="Time limit (seconds)" id="time-limit">
          <input
            type="number"
            className="input"
            aria-label="Time limit (seconds)"
            value={form.timerSeconds}
            min={1}
            onChange={e => setForm(f => ({ ...f, timerSeconds: e.target.value }))}
          />
        </FieldGroup>
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'layout', activeStep)}>
        {/* The grid is the ground floor when the game stacks multiple levels. */}
        <FieldGroup title={isMultiLevel ? 'Ground Floor Grid' : 'Grid'} id="grid">
          <MazeGenerationFields value={form.generation} onChange={patchGeneration} />
        </FieldGroup>
        {/* Level progression only applies to a multi-level game. */}
        {isMultiLevel && (
          <FieldGroup title="Levels" id="levels">
            <LevelProgressionFields value={form.levels} onChange={patchLevels} />
          </FieldGroup>
        )}
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'scene', activeStep)}>
        <GameSceneFields
          scene={form.scene}
          onSceneChange={patchScene}
          decor={form.decor}
          onDecorChange={patchDecor}
          top={form.levels.top}
          onTopChange={top => patchLevels({ top })}
          multiLevel={isMultiLevel}
        />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'objects', activeStep)}>
        {/* Each object kind grouped: its count (generation slice) next to its
            style (objects slice). */}
        <ObjectGroupsFields
          counts={form.generation}
          onCountsChange={patchGeneration}
          styles={form.objects}
          onStylesChange={patchObjects}
        />
        {/* The interim finish rig between levels — only for a multi-level game. */}
        {isMultiLevel && (
          <FieldGroup title="Levels" id="objects-levels">
            <label className="modal-stacked-input">
              Finish Cell
              <select
                className="input"
                value={form.levels.finishType}
                onChange={e => patchLevels({ finishType: e.target.value as FinishType })}
              >
                {FINISH_TYPES.map(f => (
                  <option key={f} value={f}>{titleCaseWire(f)}</option>
                ))}
              </select>
            </label>
          </FieldGroup>
        )}
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'advanced', activeStep)}>
        <AdvancedFields
          value={{
            maxHp: form.maxHp,
            enemyMovePeriodMs: form.enemyMovePeriodMs,
            minimapCellPx: form.minimapCellPx,
            minimapRadius: form.minimapRadius,
            title: form.title,
            mode: form.mode,
          }}
          onChange={patchAdvanced}
          namePlaceholder={form.name}
          // Slotted between Health & Enemies and Minimap; multi-level only.
          levelsGroup={isMultiLevel && (
            <FieldGroup title="Levels" id="advanced-levels">
              <LevelSettingsFields value={form.levels} onChange={patchLevels} />
            </FieldGroup>
          )}
        />
      </div>
    </StepModalShell>
  )
}
