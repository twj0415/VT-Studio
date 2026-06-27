import type { StoryboardItemDto } from './types'
import { hasValidSelectedImage, hasValidSelectedVideoSegment } from './reset'

export type StoryboardImageEntryField = 'sourceText'
export type StoryboardVideoEntryField = 'selectedImageId' | 'staleSelectedImage'
export type StoryboardCompositionEntryField = 'selectedVideoSegmentId' | 'staleSelectedVideoSegment'

export interface StoryboardValidationContext {
  validCharacterIds?: Iterable<string>
  validLocationIds?: Iterable<string>
}

export interface StoryboardImageEntryIssue {
  itemId: string
  index: number
  fields: StoryboardImageEntryField[]
}

export interface StoryboardVideoEntryIssue {
  itemId: string
  index: number
  fields: StoryboardVideoEntryField[]
}

export interface StoryboardCompositionEntryIssue {
  itemId: string
  index: number
  fields: StoryboardCompositionEntryField[]
}

export function validateStoryboardItemsForImageGeneration(items: StoryboardItemDto[], context: StoryboardValidationContext = {}): StoryboardImageEntryIssue[] {
  void context

  return items
    .map((item) => {
      const fields: StoryboardImageEntryField[] = []

      if (!item.sourceText.trim() && !item.narrationText.trim()) fields.push('sourceText')

      return {
        itemId: item.itemId,
        index: item.index,
        fields,
      }
    })
    .filter((issue) => issue.fields.length > 0)
}

export function validateStoryboardItemsForVideoGeneration(items: StoryboardItemDto[]): StoryboardVideoEntryIssue[] {
  return items
    .map((item) => {
      const fields: StoryboardVideoEntryField[] = []

      if (!item.selectedImageId) fields.push('selectedImageId')
      else if (!hasValidSelectedImage(item)) fields.push('staleSelectedImage')

      return {
        itemId: item.itemId,
        index: item.index,
        fields,
      }
    })
    .filter((issue) => issue.fields.length > 0)
}

export function validateStoryboardItemsForComposition(items: StoryboardItemDto[]): StoryboardCompositionEntryIssue[] {
  return items
    .map((item) => {
      const fields: StoryboardCompositionEntryField[] = []

      if (!item.selectedVideoSegmentId) fields.push('selectedVideoSegmentId')
      else if (!hasValidSelectedVideoSegment(item)) fields.push('staleSelectedVideoSegment')

      return {
        itemId: item.itemId,
        index: item.index,
        fields,
      }
    })
    .filter((issue) => issue.fields.length > 0)
}
