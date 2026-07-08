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
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'
import { modalTabPanelProps, type WizardStep } from '../utils/modalTabs'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { validateMazeGenerationFields } from '../utils/validation'
import { buildDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import type { GameDefinitionRequest } from '../types/api'

// The game-definition editor: the definition's own Details (name + description)
// plus the shared generation / scene / objects / decor field-groups, hosted in
// the dual-mode step shell — a wizard for creating a definition, tabs for
// editing one. It owns the working form state and hands the caller a finished
// `GameDefinitionRequest`, so create and edit differ only in `mode`, the seed
// state and what the caller does with the request.

const STEPS = [
  { id: 'details', label: 'Details' },
  { id: 'generation', label: 'Generation' },
  { id: 'scene', label: 'Scene' },
  { id: 'objects', label: 'Objects' },
  { id: 'decor', label: 'Decor' },
  { id: 'levels', label: 'Levels' },
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
  const [activeStep, setActiveStep] = useState<EditorStep>('details')
  const [form, setForm] = useState<DefinitionFormState>(initialForm)

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
      <div {...modalTabPanelProps(ID_PREFIX, 'details', activeStep)}>
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
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'generation', activeStep)}>
        <MazeGenerationFields value={form.generation} onChange={patchGeneration} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'scene', activeStep)}>
        <SceneFields value={form.scene} onChange={patchScene} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'objects', activeStep)}>
        <ObjectsFields value={form.objects} onChange={patchObjects} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'decor', activeStep)}>
        <DecorFields value={form.decor} onChange={patchDecor} />
      </div>

      <div {...modalTabPanelProps(ID_PREFIX, 'levels', activeStep)}>
        <LevelsFields value={form.levels} onChange={patchLevels} />
      </div>
    </StepModalShell>
  )
}
