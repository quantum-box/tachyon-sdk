import { describe, it, expect, vi, beforeEach } from "vitest";
import { AuthOperations } from "../operations/auth.js";
import type { UserProfile } from "../types.js";
import { UserRole } from "../types.js";

const mockUser: UserProfile = {
  id: "user-1",
  email: "test@example.com",
  name: "Test User",
  username: "testuser",
  emailVerified: null,
  image: null,
  role: UserRole.GENERAL,
  tenantIdList: ["tenant-1"],
  createdAt: "2024-01-01T00:00:00Z",
  updatedAt: "2024-01-01T00:00:00Z",
};

function makeClient(returnData: Record<string, unknown>) {
  return {
    query: vi.fn().mockResolvedValue(returnData),
    mutate: vi.fn().mockResolvedValue(returnData),
  };
}

describe("AuthOperations", () => {
  describe("signInWithPlatform", () => {
    it("should return AuthResult with user", async () => {
      const client = makeClient({ signInWithPlatform: mockUser });
      const auth = new AuthOperations(client);

      const result = await auth.signInWithPlatform({
        platformId: "github",
        accessToken: "gho_token123",
      });

      expect(result).toEqual({ user: mockUser });
      expect(client.mutate).toHaveBeenCalledWith(
        expect.stringContaining("signInWithPlatform"),
        expect.objectContaining({ platformId: "github", accessToken: "gho_token123" }),
      );
    });

    it("should pass allowSignUp flag", async () => {
      const client = makeClient({ signInWithPlatform: mockUser });
      const auth = new AuthOperations(client);

      await auth.signInWithPlatform({
        platformId: "github",
        accessToken: "token",
        allowSignUp: true,
      });

      expect(client.mutate).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({ allowSignUp: true }),
      );
    });
  });

  describe("verify", () => {
    it("should verify token and return AuthResult", async () => {
      const client = makeClient({ verify: mockUser });
      const auth = new AuthOperations(client);

      const result = await auth.verify("some-jwt-token");

      expect(result).toEqual({ user: mockUser });
      expect(client.mutate).toHaveBeenCalledWith(
        expect.stringContaining("verify"),
        { token: "some-jwt-token" },
      );
    });
  });

  describe("me", () => {
    it("should return the authenticated user profile", async () => {
      const client = makeClient({ me: mockUser });
      const auth = new AuthOperations(client);

      const result = await auth.me();

      expect(result).toEqual(mockUser);
      expect(client.query).toHaveBeenCalledWith(
        expect.stringContaining("me"),
      );
    });
  });

  describe("updateProfile", () => {
    it("should update and return the user profile", async () => {
      const updated = { ...mockUser, name: "Updated Name", email: "new@example.com" };
      const client = makeClient({ updateUser: updated });
      const auth = new AuthOperations(client);

      const result = await auth.updateProfile({
        id: "user-1",
        name: "Updated Name",
        email: "new@example.com",
      });

      expect(result).toEqual(updated);
      expect(client.mutate).toHaveBeenCalledWith(
        expect.stringContaining("updateUser"),
        expect.objectContaining({
          input: expect.objectContaining({ id: "user-1", name: "Updated Name" }),
        }),
      );
    });
  });

  describe("createUser", () => {
    it("should create and return a new user", async () => {
      const client = makeClient({ createUser: mockUser });
      const auth = new AuthOperations(client);

      const result = await auth.createUser({
        operatorId: "op-1",
        username: "testuser",
        email: "test@example.com",
        password: "secret123",
      });

      expect(result).toEqual(mockUser);
      expect(client.mutate).toHaveBeenCalledWith(
        expect.stringContaining("createUser"),
        expect.objectContaining({
          input: expect.objectContaining({
            operatorId: "op-1",
            username: "testuser",
            email: "test@example.com",
          }),
        }),
      );
    });
  });
});
