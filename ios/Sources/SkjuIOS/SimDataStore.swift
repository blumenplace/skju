import Foundation
import OSLog

private extension Logger {
  static let simStore = Logger(subsystem: Bundle.main.bundleIdentifier ?? "SkjuIOS", category: "SimDataStore")
}

/// A lightweight HTTP-backed store for `SensorItem` using JSON.
///
/// Note: This is a networking store, not (yet) wired as a SwiftData `DataStore` implementation.
/// It provides async methods to load, insert, update and delete `SensorItem` on a remote service.
/// If you want a true SwiftData custom `DataStore` (iOS 18+ API), this class can be used under the hood.
public actor SimDataStore {
  public struct SensorDTO: Codable, Sendable, Equatable, Hashable {
    public let id: UUID
    public let x: Double
    public let y: Double
  }

  public enum Error: Swift.Error {
    case badURL
    case server(status: Int)
    case decodingFailed(Swift.Error)
    case encodingFailed(Swift.Error)
    case transport(Swift.Error)
  }

  public var baseURL: URL
  private let session: URLSession
  private let jsonEncoder: JSONEncoder
  private let jsonDecoder: JSONDecoder

  public init(baseURL: URL = URL(string: "https://skju-sim.blumen.place")!, session: URLSession = .shared) {
    self.baseURL = baseURL
    self.session = session
    self.jsonEncoder = JSONEncoder()
    self.jsonDecoder = JSONDecoder()
  }

  public func loadAll() async throws -> [SensorItem] {
    let dtos: [SensorDTO] = try await get(path: "/sensors")
    return dtos.map { SensorItem(id: $0.id, x: $0.x, y: $0.y) }
  }

  /// Synchronize local changes with the server.
  /// - Parameters:
  ///   - inserted: items to create remotely
  ///   - updated: items to update remotely
  ///   - deleted: items to delete remotely
  public func save(inserted: [SensorItem], updated: [SensorItem], deleted: [SensorItem]) async throws {
    // Insert
    if !inserted.isEmpty {
      let payload = inserted.map { SensorDTO(id: $0.id, x: $0.x, y: $0.y) }
      try await post(path: "/sensors", body: payload)
    }

    // Update
    for item in updated {
      let dto = SensorDTO(id: item.id, x: item.x, y: item.y)
      try await put(path: "/sensors/\(dto.id.uuidString)", body: dto)
    }

    // Delete
    for item in deleted {
      try await delete(path: "/sensors/\(item.id.uuidString)")
    }
  }

  private func get<T: Decodable>(path: String) async throws -> T {
    let url = baseURL.appendingPathComponent(path.trimmingCharacters(in: CharacterSet(charactersIn: "/")))
    var req = URLRequest(url: url)
    req.httpMethod = "GET"
    req.setValue("application/json", forHTTPHeaderField: "Accept")
    do {
      let (data, resp) = try await session.data(for: req)
      guard let http = resp as? HTTPURLResponse else { throw Error.transport(URLError(.badServerResponse)) }
      guard (200..<300).contains(http.statusCode) else {
        Logger.simStore.error("GET failed: status=\(http.statusCode)")
        throw Error.server(status: http.statusCode)
      }
      do { return try jsonDecoder.decode(T.self, from: data) }
      catch { throw Error.decodingFailed(error) }
    } catch { throw Error.transport(error) }
  }

  private func post<T: Encodable>(path: String, body: T) async throws {
    try await sendWithBody(path: path, method: "POST", body: body)
  }

  private func put<T: Encodable>(path: String, body: T) async throws {
    try await sendWithBody(path: path, method: "PUT", body: body)
  }

  private func delete(path: String) async throws {
    let url = baseURL.appendingPathComponent(path.trimmingCharacters(in: CharacterSet(charactersIn: "/")))
    var req = URLRequest(url: url)
    req.httpMethod = "DELETE"
    req.setValue("application/json", forHTTPHeaderField: "Accept")
    do {
      let (_, resp) = try await session.data(for: req)
      guard let http = resp as? HTTPURLResponse else { throw Error.transport(URLError(.badServerResponse)) }
      guard (200..<300).contains(http.statusCode) else {
        Logger.simStore.error("DELETE failed: status=\(http.statusCode)")
        throw Error.server(status: http.statusCode)
      }
    } catch { throw Error.transport(error) }
  }

  private func sendWithBody<T: Encodable>(path: String, method: String, body: T) async throws {
    let url = baseURL.appendingPathComponent(path.trimmingCharacters(in: CharacterSet(charactersIn: "/")))
    var req = URLRequest(url: url)
    req.httpMethod = method
    req.setValue("application/json", forHTTPHeaderField: "Content-Type")
    req.setValue("application/json", forHTTPHeaderField: "Accept")
    do {
      req.httpBody = try jsonEncoder.encode(body)
    } catch {
      throw Error.encodingFailed(error)
    }
    do {
      let (_, resp) = try await session.data(for: req)
      guard let http = resp as? HTTPURLResponse else { throw Error.transport(URLError(.badServerResponse)) }
      guard (200..<300).contains(http.statusCode) else {
        Logger.simStore.error("\(method) failed: status=\(http.statusCode)")
        throw Error.server(status: http.statusCode)
      }
    } catch { throw Error.transport(error) }
  }
}
