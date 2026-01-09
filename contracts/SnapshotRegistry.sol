// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title ArchiveStream Snapshot Registry
 * @dev Immutable registry of web snapshots with cryptographic proofs
 */
contract SnapshotRegistry {
    struct Snapshot {
        string url;
        uint256 timestamp;
        bytes32 contentHash;  // SHA-256 of WARC content
        string ipfsCid;       // IPFS content identifier
        address archiver;     // Who archived it
        uint256 blockNumber;  // When it was registered
    }
    
    // Mapping from snapshot ID to snapshot data
    mapping(bytes32 => Snapshot) public snapshots;
    
    // Mapping from URL to list of snapshot IDs
    mapping(string => bytes32[]) public urlSnapshots;
    
    // Total number of snapshots
    uint256 public totalSnapshots;
    
    // Events
    event SnapshotRegistered(
        bytes32 indexed snapshotId,
        string url,
        uint256 timestamp,
        string ipfsCid,
        address indexed archiver
    );
    
    event SnapshotVerified(
        bytes32 indexed snapshotId,
        address indexed verifier
    );
    
    /**
     * @dev Register a new snapshot on-chain
     * @param url The archived URL
     * @param timestamp Unix timestamp of the snapshot
     * @param contentHash SHA-256 hash of the WARC content
     * @param ipfsCid IPFS CID where content is stored
     */
    function registerSnapshot(
        string memory url,
        uint256 timestamp,
        bytes32 contentHash,
        string memory ipfsCid
    ) public returns (bytes32) {
        // Generate unique snapshot ID
        bytes32 snapshotId = keccak256(
            abi.encodePacked(url, timestamp, contentHash)
        );
        
        // Ensure snapshot doesn't already exist
        require(snapshots[snapshotId].timestamp == 0, "Snapshot already exists");
        
        // Store snapshot
        snapshots[snapshotId] = Snapshot({
            url: url,
            timestamp: timestamp,
            contentHash: contentHash,
            ipfsCid: ipfsCid,
            archiver: msg.sender,
            blockNumber: block.number
        });
        
        // Add to URL index
        urlSnapshots[url].push(snapshotId);
        totalSnapshots++;
        
        emit SnapshotRegistered(snapshotId, url, timestamp, ipfsCid, msg.sender);
        
        return snapshotId;
    }
    
    /**
     * @dev Verify a snapshot's integrity
     * @param snapshotId The snapshot to verify
     * @param contentHash The hash to verify against
     */
    function verifySnapshot(bytes32 snapshotId, bytes32 contentHash) public returns (bool) {
        Snapshot memory snapshot = snapshots[snapshotId];
        require(snapshot.timestamp != 0, "Snapshot does not exist");
        
        bool isValid = snapshot.contentHash == contentHash;
        
        if (isValid) {
            emit SnapshotVerified(snapshotId, msg.sender);
        }
        
        return isValid;
    }
    
    /**
     * @dev Get all snapshots for a URL
     * @param url The URL to query
     */
    function getSnapshotsForUrl(string memory url) public view returns (bytes32[] memory) {
        return urlSnapshots[url];
    }
    
    /**
     * @dev Get snapshot details
     * @param snapshotId The snapshot ID
     */
    function getSnapshot(bytes32 snapshotId) public view returns (
        string memory url,
        uint256 timestamp,
        bytes32 contentHash,
        string memory ipfsCid,
        address archiver,
        uint256 blockNumber
    ) {
        Snapshot memory snapshot = snapshots[snapshotId];
        require(snapshot.timestamp != 0, "Snapshot does not exist");
        
        return (
            snapshot.url,
            snapshot.timestamp,
            snapshot.contentHash,
            snapshot.ipfsCid,
            snapshot.archiver,
            snapshot.blockNumber
        );
    }
}
