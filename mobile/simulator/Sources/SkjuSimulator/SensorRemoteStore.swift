import Foundation
import OSLog
import SwiftData

public struct SensorsStoreConfiguration: DataStoreConfiguration, Hashable, Sendable {
  public typealias Store = SensorsRemoteStore

  public var schema: Schema?
  public var name: String
  public let baseURL: URL

  public init(name: String = "SensorsRemoteStore", schema: Schema, baseURL: URL = URL(string: "https://skju-sim.blumen.place")!) {
    self.name = name
    self.schema = schema
    self.baseURL = baseURL
  }
}

public struct SensorSnapshot: DataStoreSnapshot {
  public let persistentIdentifier: PersistentIdentifier
  public var id: UUID
  public var x: Double
  public var y: Double

  public init(persistentIdentifier: PersistentIdentifier, id: UUID, x: Double, y: Double) {
    self.persistentIdentifier = persistentIdentifier
    self.id = id
    self.x = x
    self.y = y
  }

  public init(from backing: any BackingData, relatedBackingDatas: inout [PersistentIdentifier: any BackingData]) {
    let pid = backing.persistentModelID!
    let id: UUID = backing.getValue(forKey: \SensorItem.id)
    let x: Double = backing.getValue(forKey: \SensorItem.x)
    let y: Double = backing.getValue(forKey: \SensorItem.y)
    self.init(persistentIdentifier: pid, id: id, x: x, y: y)
  }

  public func copy(persistentIdentifier: PersistentIdentifier, remappedIdentifiers: [PersistentIdentifier : PersistentIdentifier]?) -> SensorSnapshot {
    SensorSnapshot(persistentIdentifier: persistentIdentifier, id: id, x: x, y: y)
  }
}

public final class SensorsRemoteStore: DataStore {
  public typealias Configuration = SensorsStoreConfiguration
  public typealias Snapshot = SensorSnapshot

  public let identifier: String
  public let schema: Schema
  public let configuration: Configuration

  private let sim: SimDataStore

  public required init(_ configuration: Configuration, migrationPlan: SchemaMigrationPlan.Type?) throws {
    self.configuration = configuration
    self.identifier = configuration.identifier
    self.schema = Schema([SensorItem.self])
    self.sim = awaitSimStore(baseURL: configuration.baseURL)
  }

  // Helper to construct the actor synchronously
  private static func awaitSimStore(baseURL: URL) -> SimDataStore {
    // Actor initializers are synchronous; just return
    return SimDataStore(baseURL: baseURL)
  }

  public func erase() throws {
    // No bulk erase endpoint; ignore for now.
  }

  public func fetch<T>(_ request: DataStoreFetchRequest<T>) throws -> DataStoreFetchResult<T, Snapshot> where T : PersistentModel {
    guard T.self == SensorItem.self else {
      return DataStoreFetchResult(descriptor: request.descriptor, fetchedSnapshots: [], relatedSnapshots: [:])
    }

    // Load synchronously by bridging async call
    var fetchResult: Result<[Snapshot], Error>!
    let sem = DispatchSemaphore(value: 0)
    Task {
      do {
        let items = try await self.sim.loadAll()
        let snaps: [Snapshot] = items.map { item in
          let pid = PersistentIdentifier(rawValue: item.id.uuidString)
          return Snapshot(persistentIdentifier: pid, id: item.id, x: item.x, y: item.y)
        }
        fetchResult = .success(snaps)
      } catch {
        fetchResult = .failure(error)
      }
      sem.signal()
    }
    sem.wait()
    let snapshots = try fetchResult.get()

    let offset = request.descriptor.fetchOffset
    let limit = request.descriptor.fetchLimit
    let sliced = Array(snapshots.dropFirst(offset).prefix(limit == 0 ? Int.max : limit))

    return DataStoreFetchResult(descriptor: request.descriptor, fetchedSnapshots: sliced, relatedSnapshots: [:])
  }

  public func fetchCount<T>(_ request: DataStoreFetchRequest<T>) throws -> Int where T : PersistentModel {
    try fetch(request).fetchedSnapshots.count
  }

  public func fetchIdentifiers<T>(_ request: DataStoreFetchRequest<T>) throws -> [PersistentIdentifier] where T : PersistentModel {
    try fetch(request).fetchedSnapshots.map { $0.persistentIdentifier }
  }

  public func save(_ request: DataStoreSaveChangesRequest<Snapshot>) throws -> DataStoreSaveChangesResult<Snapshot> {
    // Bridge async
    var saveError: Error?
    let sem = DispatchSemaphore(value: 0)
    Task {
      do {
        let inserted = request.inserted.map { SensorItem(id: $0.id, x: $0.x, y: $0.y) }
        let updated = request.updated.map { SensorItem(id: $0.id, x: $0.x, y: $0.y) }
        let deleted = request.deleted.map { SensorItem(id: $0.id, x: $0.x, y: $0.y) }
        try await self.sim.save(inserted: inserted, updated: updated, deleted: deleted)
      } catch {
        saveError = error
      }
      sem.signal()
    }
    sem.wait()
    if let err = saveError { throw err }
    return DataStoreSaveChangesResult(for: identifier)
  }

//  public func initializeState(for editingState: EditingState) { }
//
//  public func invalidateState(for editingState: EditingState) { }
//
//  public func cachedSnapshots(for persistentIdentifiers: [PersistentIdentifier], editingState: EditingState) throws -> [PersistentIdentifier : Snapshot] {
//    [:]
//  }
}
