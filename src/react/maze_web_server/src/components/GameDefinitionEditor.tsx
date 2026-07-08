import { useState } from 'react'
import { StepModalShell } from './StepModalShell'
import { MazeGenerationFields, type MazeGenerationFieldsValue } from './MazeGenerationFields'
import {
  SceneFields,
  ObjectsFields,
  DecorFields,
  type SceneFieldsValue,
  type ObjectsFieldsValue,
  type DecorFieldsValue,
} from './GameSettingsFields'
import { LevelsFields } from './LevelsFields'
import { AdvancedFields, type AdvancedFieldsValue } from './AdvancedFields'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'
import { modalTabPanelProps, type WizardStep } from '../utils/modalTabs'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { validateMazeGenerationFields } from '../utils/validation'
import { MAX_LEVEL_COUNT } from '../utils/gameDefinitions'
import { buildDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import type { GameDefinitionRequest } from '../types/api'

// The game-definition editor: the definition's own General details (name,
// description, level count + time limit) plus the shared generation / scene /
// objects / decor / levels / advanced field-groups, hosted in the dual-mode step
// shell — a wizard for creating a definition, tabs for editing one. It owns the
// working form state and hands the caller a finished `GameDefinitionRequest`, so
// create and edit differ only in `mode`, the seed state and what the caller does
// with the request.

const STEPS = [
  { id: 'general', label: 'General' },
  { id: 'generation', label: 'Generation' },
  { id: 'scene', label: 'Scene' },
  { id: 'objects', label: 'Objects' },
  { id: 'decor', label: 'Decor' },
  { id: 'levels', label: 'Levels' },
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

  // The Levels tab only exists for a multi-level game — a single-level game
  // (count ≤ 1, incl. blank/invalid) hides it from both the wizard rail and the
  // tab strip, so the single-vs-multi-level decision (the count on General) is
  // what reveals it. The Levels panel itself stays mounted below, so toggling
  // the count down and back up preserves the level settings.
  const isMultiLevel = parseInt(form.levels.count, 10) > 1
  const visibleSteps = isMultiLevel ? STEPS : STEPS.filter(s => s.id !== 'levels')
  // Guard the shell against a stale active step (e.g. Levels was active when the
  // count dropped to 1); fall back to General, which always exists.
  const effectiveStep = visibleSteps.some(s => s.id === activeStep) ? activeStep : 'general'

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
      steps={visibleSteps}
      activeStep={effectiveStep}
      onStepChange={setActiveStep}
      idPrefix={ID_PREFIX}
      ariaLabel="Game definition steps"
      onCancel={onCancel}
      onCommit={handleCommit}
      canCommit={canCommit}
      commitLabel={commitLabel}
      footerNote={generationError && <p role="alert" className="error-msg">{generationError}</p>}
    >
      <div {...modalTabPanelProps(ID_PREFIX, 'general', effectiveStep)}>
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
        <label className="modal-stacked-input">
          Levels
          <input
            type="number"
            className="input"
            value={form.levels.count}
            min={1}
            max={MAX_LEVEL_COUNT}
            onChange={e => patchLevels({ count: e.target.value })}
          />
        </label>
        <label className="modal-stacked-input">
          Time limit (seconds)
          <input
            type="number"
            className="input"
            value={form.timerSeconds}
            min={1}
            onChange={e => setForm(f => ({ ...f, timerSeconds: e.target.value }))}
          />
        </label>
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'generation', effectiveStep)}>
        <MazeGenerationFields value={form.generation} onChange={patchGeneration} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'scene', effectiveStep)}>
        <SceneFields value={form.scene} onChange={patchScene} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'objects', effectiveStep)}>
        <ObjectsFields value={form.objects} onChange={patchObjects} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'decor', effectiveStep)}>
        <DecorFields value={form.decor} onChange={patchDecor} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'levels', effectiveStep)}>
        <LevelsFields value={form.levels} onChange={patchLevels} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'advanced', effectiveStep)}>
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
        />
      </div>
    </StepModalShell>
  )
}
