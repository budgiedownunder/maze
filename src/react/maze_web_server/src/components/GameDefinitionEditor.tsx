import { useMemo, useState } from 'react'
import { StepModalShell } from './StepModalShell'
import { GameImageEditor } from './GameImageEditor'
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
import { ConfirmModal } from './ConfirmModal'
import { AdvancedFields, type AdvancedFieldsValue } from './AdvancedFields'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'
import { modalTabPanelProps, type WizardStep } from '../utils/modalTabs'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { MAX_GAME_MAZE_DIMENSION, validateMazeGenerationFields } from '../utils/validation'
import { MAX_LEVEL_COUNT, FINISH_TYPES, ROTATIONS, isGameplayChange, reshuffleConfirmMessage, rotationLabel, rotationDescription, type FinishType, type Rotation } from '../utils/gameDefinitions'
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
  { id: 'layout', label: 'Layout' },
  { id: 'scene', label: 'Scene' },
  { id: 'objects', label: 'Objects' },
  { id: 'advanced', label: 'Advanced' },
] as const satisfies readonly WizardStep[]

type EditorStep = (typeof STEPS)[number]['id']

const ID_PREFIX = 'gamedef'

// Fills the in-game splash title / status-bar label from the name when left
// blank, so a definition always announces itself as something.
function withNameDefaults(form: DefinitionFormState): DefinitionFormState {
  const name = form.name.trim()
  return {
    ...form,
    name,
    title: form.title.trim() === '' ? name : form.title,
    mode: form.mode.trim() === '' ? name : form.mode,
  }
}

interface GameDefinitionEditorProps {
  mode: 'tabs' | 'wizard'
  /** Seed state — the defaults for a new definition, or a parsed stored one. */
  initialForm: DefinitionFormState
  title: string
  /** Commit-button label; defaults to the shell's Finish (wizard) / Save (tabs). */
  commitLabel?: string
  onSubmit: (request: GameDefinitionRequest) => void
  onCancel: () => void
  /**
   * Persists a reshuffle (server re-mints the seed) and resolves the new seed.
   * When provided — i.e. editing an existing definition — the Advanced tab
   * offers a "Reshuffle layout" action behind a confirm dialog. Omitted for a
   * brand-new definition (there is no record to reshuffle yet).
   */
  onReshuffle?: () => Promise<number>
  /** Whether the definition already has leaderboard scores — drives the stronger
   *  reshuffle-confirm wording (the board will be wiped). */
  hasScores?: boolean
  /**
   * Launches a one-off preview of the given (in-progress) config. When provided,
   * a footer Preview button appears, enabled once the generation config is valid
   * (a name is NOT required — unlike Finish — since a preview is not saved).
   */
  onPreview?: (config: GameDefinitionRequest['config']) => void
  /**
   * Image control for the saved game. Present only when editing an existing
   * definition (a brand-new game has no id to attach an upload to); its Details
   * tab then shows a preview + Change/Remove. The image is a separate resource
   * (uploaded/removed immediately), so it is independent of this form's Save.
   * `onImageChange` reports the new marker so the workshop row can refresh.
   */
  image?: { id: string; imageUpdatedAt?: string | null; onImageChange: (imageUpdatedAt: string | null) => void }
}

export function GameDefinitionEditor({
  mode,
  initialForm,
  title,
  commitLabel,
  onSubmit,
  onCancel,
  onReshuffle,
  hasScores = false,
  onPreview,
  image,
}: GameDefinitionEditorProps) {
  const { max_maze_cells } = useAppFeatures()
  const [activeStep, setActiveStep] = useState<EditorStep>('general')
  const [form, setForm] = useState<DefinitionFormState>(initialForm)

  // Reshuffle confirm-dialog state (only reachable when `onReshuffle` is set).
  const [showReshuffleConfirm, setShowReshuffleConfirm] = useState(false)
  const [isReshuffling, setIsReshuffling] = useState(false)
  const [reshuffleError, setReshuffleError] = useState<string | null>(null)

  async function handleReshuffle() {
    if (!onReshuffle) return
    setIsReshuffling(true)
    setReshuffleError(null)
    try {
      const newSeed = await onReshuffle()
      // Keep the (hidden) seed in the form in step with the server's new one.
      setForm(f => ({ ...f, seed: newSeed }))
      setShowReshuffleConfirm(false)
    } catch (ex: unknown) {
      setReshuffleError((ex as { message?: string }).message ?? 'Failed to reshuffle.')
    } finally {
      setIsReshuffling(false)
    }
  }

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
  // Preview only needs a generatable config — a name is a save-only requirement.
  const canPreview = generationError === null

  // In edit (tabs) mode, only offer Save when the form actually changed. The
  // image and seed (reshuffle) are separate, immediately-persisted resources, so
  // they don't count as unsaved form changes — compare the editable form holding
  // the seed constant. In wizard (create) mode there is always something to do.
  const isEdit = mode === 'tabs'
  const isDirty = useMemo(
    () => JSON.stringify({ ...form, seed: 0 }) !== JSON.stringify({ ...initialForm, seed: 0 }),
    [form, initialForm],
  )

  // A pending save awaiting confirmation because it changes gameplay and would
  // reset a scored board. Null when no confirmation is needed.
  const [pendingSave, setPendingSave] = useState<GameDefinitionRequest | null>(null)

  // Build the config (+ request) from the live form, defaulting the in-game
  // splash title / status-bar label from the name when left blank, so a
  // definition always announces itself as something.
  function buildFromForm() {
    return buildDefinitionConfig(withNameDefaults(form))
  }

  function handleCommit() {
    const request = buildFromForm().request
    // A gameplay-affecting edit on a scored game wipes its board on save, so
    // warn first; cosmetic-only edits (and unscored games) save straight away.
    const before = buildDefinitionConfig(withNameDefaults(initialForm)).request.config
    const changesGameplay = initialForm.rotation !== form.rotation || isGameplayChange(before, request.config)
    if (hasScores && changesGameplay) {
      setPendingSave(request)
    } else {
      onSubmit(request)
    }
  }

  function handlePreview() {
    // `request.config` is the opaque, widened config (the StartConfig blob).
    onPreview?.(buildFromForm().request.config)
  }

  const reshuffleMessage = reshuffleConfirmMessage(hasScores)

  return (
    <>
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
      showCommit={!isEdit || isDirty}
      cancelLabel={isEdit && !isDirty ? 'Close' : 'Cancel'}
      commitLabel={commitLabel}
      onPreview={onPreview && handlePreview}
      canPreview={canPreview}
      footerNote={generationError && <p role="alert" className="error-msg">{generationError}</p>}
    >
      <div {...modalTabPanelProps(ID_PREFIX, 'general', activeStep)}>
        <FieldGroup title="Details" id="details">
          {image && (
            <GameImageEditor
              kind="definition"
              id={image.id}
              imageUpdatedAt={image.imageUpdatedAt}
              onChange={image.onImageChange}
            />
          )}
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

        <FieldGroup title="Rotation" id="rotation">
          <select
            className="input"
            aria-label="Rotation"
            value={form.rotation}
            onChange={e => setForm(f => ({ ...f, rotation: e.target.value as Rotation }))}
          >
            {ROTATIONS.map(r => <option key={r} value={r}>{rotationLabel(r)}</option>)}
          </select>
          <p className="access-tier-desc">{rotationDescription(form.rotation)}</p>
        </FieldGroup>

        {/* Single-field groups: the group heading is the field's label, so the
            lone input carries an aria-label for its accessible name. */}
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
        {/* Number of stacked levels — drives the multi-level layout controls
            below (and the single-field group's heading is its label). */}
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
        {/* The grid is the ground floor when the game stacks multiple levels. */}
        <FieldGroup title={isMultiLevel ? 'Ground Floor Grid' : 'Grid'} id="grid">
          <MazeGenerationFields
            value={form.generation}
            onChange={patchGeneration}
            maxDimension={MAX_GAME_MAZE_DIMENSION}
          />
        </FieldGroup>
        {/* Level progression only applies to a multi-level game. */}
        {isMultiLevel && (
          <FieldGroup title="Levels" id="levels">
            <LevelProgressionFields value={form.levels} onChange={patchLevels} />
          </FieldGroup>
        )}
        {/* Reshuffle re-generates the layout (server-side); offered only when
            editing an existing definition. The seed stays hidden. */}
        {onReshuffle && (
          <button
            type="button"
            className="btn-danger"
            onClick={() => { setShowReshuffleConfirm(true); setReshuffleError(null) }}
          >
            Reshuffle Layout
          </button>
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

    {showReshuffleConfirm && (
      <ConfirmModal
        title="Reshuffle Layout"
        message={reshuffleMessage}
        confirmLabel="Reshuffle"
        isDangerous={hasScores}
        isLoading={isReshuffling}
        error={reshuffleError}
        onConfirm={() => void handleReshuffle()}
        onCancel={() => { setShowReshuffleConfirm(false); setReshuffleError(null) }}
      />
    )}

    {pendingSave && (
      <ConfirmModal
        title="Save Changes"
        message="These changes affect how the game plays, so its leaderboard will be reset — the current scores were set on the previous version. This can't be undone."
        confirmLabel="Save and reset"
        isDangerous
        onConfirm={() => { const request = pendingSave; setPendingSave(null); onSubmit(request) }}
        onCancel={() => setPendingSave(null)}
      />
    )}
    </>
  )
}
