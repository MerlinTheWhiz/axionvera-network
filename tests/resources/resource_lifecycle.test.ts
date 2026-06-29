import { describe, it, expect, beforeEach } from 'vitest';

// ===========================================================================
// RESOURCE LIFECYCLE STATE DEFINITIONS (Mirrors contracts/resources/src/lib.rs)
// ===========================================================================

export enum ResourceState {
  Created = 0,
  Active = 1,
  Suspended = 2,
  Archived = 3,
  Retired = 4,
}

// ===========================================================================
// RESOURCE EVENT INTERFACES
// ===========================================================================

export interface ResourceLifecycleEvent {
  event_version: number;
  resource_id: string;
  old_state: string;
  new_state: string;
  caller: string;
  timestamp: number;
}

export interface ResourceInfo {
  id: string;
  state: ResourceState;
  created_at: number;
  updated_at: number;
}

// ===========================================================================
// RESOURCE LIFECYCLE SIMULATOR (Mirrors Rust validation rules & event emission)
// ===========================================================================

class ResourceLifecycleManager {
  private resources: Map<string, ResourceState> = new Map();
  public events: ResourceLifecycleEvent[] = [];

  private emitEvent(
    resourceId: string,
    oldState: string,
    newState: string,
    caller: string
  ) {
    this.events.push({
      event_version: 1,
      resource_id: resourceId,
      old_state: oldState,
      new_state: newState,
      caller,
      timestamp: Date.now(),
    });
  }

  createResource(resourceId: string, caller: string): ResourceState {
    if (this.resources.has(resourceId)) {
      throw new Error('ResourceError: AlreadyExists');
    }
    this.resources.set(resourceId, ResourceState.Created);
    this.emitEvent(resourceId, 'None', 'Created', caller);
    return ResourceState.Created;
  }

  transitionTo(
    resourceId: string,
    newState: ResourceState,
    caller: string
  ): ResourceState {
    const currentState = this.resources.get(resourceId);
    if (currentState === undefined) {
      throw new Error('ResourceError: NotFound');
    }
    if (currentState === newState) {
      throw new Error('ResourceError: AlreadyInState');
    }

    const valid =
      (currentState === ResourceState.Created && newState === ResourceState.Active) ||
      (currentState === ResourceState.Active &&
        [ResourceState.Suspended, ResourceState.Archived].includes(newState)) ||
      (currentState === ResourceState.Suspended &&
        [ResourceState.Active, ResourceState.Archived].includes(newState)) ||
      (currentState === ResourceState.Archived && newState === ResourceState.Retired);

    if (!valid) {
      throw new Error('ResourceError: InvalidTransition');
    }

    const oldLabel = ResourceState[currentState];
    const newLabel = ResourceState[newState];
    this.resources.set(resourceId, newState);
    this.emitEvent(resourceId, oldLabel, newLabel, caller);
    return newState;
  }

  getState(resourceId: string): ResourceState {
    const state = this.resources.get(resourceId);
    if (state === undefined) {
      throw new Error('ResourceError: NotFound');
    }
    return state;
  }

  exists(resourceId: string): boolean {
    return this.resources.has(resourceId);
  }

  getResourceCount(): number {
    return this.resources.size;
  }

  isTerminal(state: ResourceState): boolean {
    return state === ResourceState.Retired;
  }
}

// ===========================================================================
// TESTS & ACCEPTANCE CRITERIA VALIDATION
// ===========================================================================

describe('Protocol Resource Lifecycle Manager', () => {
  let manager: ResourceLifecycleManager;
  const admin = 'GADMIN1234567890';

  beforeEach(() => {
    manager = new ResourceLifecycleManager();
  });

  describe('1. Resource Creation', () => {
    it('should create a resource in Created state', () => {
      const state = manager.createResource('res_1', admin);
      expect(state).toBe(ResourceState.Created);
      expect(manager.exists('res_1')).toBe(true);
    });

    it('should reject duplicate creation', () => {
      manager.createResource('res_1', admin);
      expect(() => manager.createResource('res_1', admin)).toThrow(
        'ResourceError: AlreadyExists'
      );
    });

    it('should emit creation event', () => {
      manager.createResource('res_1', admin);
      expect(manager.events).toHaveLength(1);
      expect(manager.events[0]).toMatchObject({
        event_version: 1,
        resource_id: 'res_1',
        old_state: 'None',
        new_state: 'Created',
        caller: admin,
      });
    });
  });

  describe('2. Complete Lifecycle Transitions', () => {
    it('should transition through the full lifecycle', () => {
      const id = 'res_full';
      manager.createResource(id, admin);

      // Created -> Active
      let state = manager.transitionTo(id, ResourceState.Active, admin);
      expect(state).toBe(ResourceState.Active);

      // Active -> Suspended
      state = manager.transitionTo(id, ResourceState.Suspended, admin);
      expect(state).toBe(ResourceState.Suspended);

      // Suspended -> Active
      state = manager.transitionTo(id, ResourceState.Active, admin);
      expect(state).toBe(ResourceState.Active);

      // Active -> Archived
      state = manager.transitionTo(id, ResourceState.Archived, admin);
      expect(state).toBe(ResourceState.Archived);

      // Archived -> Retired
      state = manager.transitionTo(id, ResourceState.Retired, admin);
      expect(state).toBe(ResourceState.Retired);

      // Terminal verification
      expect(manager.isTerminal(state)).toBe(true);
    });

    it('should allow suspend after active and resume', () => {
      const id = 'res_suspend';
      manager.createResource(id, admin);
      manager.transitionTo(id, ResourceState.Active, admin);
      manager.transitionTo(id, ResourceState.Suspended, admin);
      expect(manager.getState(id)).toBe(ResourceState.Suspended);

      // Resume
      manager.transitionTo(id, ResourceState.Active, admin);
      expect(manager.getState(id)).toBe(ResourceState.Active);
    });

    it('should allow archive from suspended state', () => {
      const id = 'res_suspend_archive';
      manager.createResource(id, admin);
      manager.transitionTo(id, ResourceState.Active, admin);
      manager.transitionTo(id, ResourceState.Suspended, admin);
      manager.transitionTo(id, ResourceState.Archived, admin);
      expect(manager.getState(id)).toBe(ResourceState.Archived);
    });
  });

  describe('3. Invalid Transition Rejection', () => {
    it('should reject Created -> Archived', () => {
      const id = 'res_inv1';
      manager.createResource(id, admin);
      expect(() => manager.transitionTo(id, ResourceState.Archived, admin)).toThrow(
        'ResourceError: InvalidTransition'
      );
    });

    it('should reject Created -> Retired (skip to terminal)', () => {
      const id = 'res_inv2';
      manager.createResource(id, admin);
      expect(() => manager.transitionTo(id, ResourceState.Retired, admin)).toThrow(
        'ResourceError: InvalidTransition'
      );
    });

    it('should reject Active -> Retired (must go through Archived)', () => {
      const id = 'res_inv3';
      manager.createResource(id, admin);
      manager.transitionTo(id, ResourceState.Active, admin);
      expect(() => manager.transitionTo(id, ResourceState.Retired, admin)).toThrow(
        'ResourceError: InvalidTransition'
      );
    });

    it('should reject Archived -> Active (no going back)', () => {
      const id = 'res_inv4';
      manager.createResource(id, admin);
      manager.transitionTo(id, ResourceState.Active, admin);
      manager.transitionTo(id, ResourceState.Archived, admin);
      expect(() => manager.transitionTo(id, ResourceState.Active, admin)).toThrow(
        'ResourceError: InvalidTransition'
      );
    });

    it('should reject transitions from terminal Retired state', () => {
      const id = 'res_inv5';
      manager.createResource(id, admin);
      manager.transitionTo(id, ResourceState.Active, admin);
      manager.transitionTo(id, ResourceState.Archived, admin);
      manager.transitionTo(id, ResourceState.Retired, admin);
      expect(() => manager.transitionTo(id, ResourceState.Active, admin)).toThrow(
        'ResourceError: InvalidTransition'
      );
    });

    it('should reject AlreadyInState', () => {
      const id = 'res_inv6';
      manager.createResource(id, admin);
      expect(() => manager.transitionTo(id, ResourceState.Created, admin)).toThrow(
        'ResourceError: AlreadyInState'
      );
    });

    it('should reject operations on non-existent resource', () => {
      expect(() => manager.getState('nonexistent')).toThrow(
        'ResourceError: NotFound'
      );
      expect(() => manager.transitionTo('nonexistent', ResourceState.Active, admin)).toThrow(
        'ResourceError: NotFound'
      );
    });
  });

  describe('4. Event Emission Verification', () => {
    it('should emit events for every transition', () => {
      const id = 'res_events';
      manager.createResource(id, admin);
      expect(manager.events).toHaveLength(1);

      manager.transitionTo(id, ResourceState.Active, admin);
      expect(manager.events).toHaveLength(2);
      expect(manager.events[1]).toMatchObject({
        old_state: 'Created',
        new_state: 'Active',
      });

      manager.transitionTo(id, ResourceState.Archived, admin);
      expect(manager.events).toHaveLength(3);
      expect(manager.events[2]).toMatchObject({
        old_state: 'Active',
        new_state: 'Archived',
      });

      manager.transitionTo(id, ResourceState.Retired, admin);
      expect(manager.events).toHaveLength(4);
      expect(manager.events[3]).toMatchObject({
        old_state: 'Archived',
        new_state: 'Retired',
      });
    });
  });

  describe('5. Query Operations', () => {
    it('should return correct count', () => {
      expect(manager.getResourceCount()).toBe(0);
      manager.createResource('res_a', admin);
      expect(manager.getResourceCount()).toBe(1);
      manager.createResource('res_b', admin);
      expect(manager.getResourceCount()).toBe(2);
    });
  });

  describe('6. Documentation & Validation Matrix Verification', () => {
    it('should have all documented state transitions working', () => {
      // Verify every valid transition path from the state diagram
      const id = 'res_doc';
      manager.createResource(id, admin);

      // Path: Created -> Active
      manager.transitionTo(id, ResourceState.Active, admin);

      // Path: Active -> Suspended -> Active
      manager.transitionTo(id, ResourceState.Suspended, admin);
      manager.transitionTo(id, ResourceState.Active, admin);

      // Path: Active -> Suspended -> Archived
      manager.transitionTo(id, ResourceState.Suspended, admin);
      manager.transitionTo(id, ResourceState.Archived, admin);

      // Reset via new resource for next path
      const id2 = 'res_doc2';
      manager.createResource(id2, admin);
      manager.transitionTo(id2, ResourceState.Active, admin);

      // Path: Active -> Archived -> Retired
      manager.transitionTo(id2, ResourceState.Archived, admin);
      manager.transitionTo(id2, ResourceState.Retired, admin);
      expect(manager.isTerminal(manager.getState(id2))).toBe(true);
    });
  });
});
