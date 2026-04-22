import type {
  AuthResult,
  UserProfile,
  SignInWithPlatformInput,
  CreateUserInput,
  UpdateProfileInput,
} from "../types.js";

interface GraphQLClient {
  query<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
  mutate<T = unknown>(document: string, variables?: Record<string, unknown>): Promise<T>;
}

const USER_FIELDS = `
  id
  email
  name
  username
  emailVerified
  image
  role
  tenantIdList
  createdAt
  updatedAt
`;

const SIGN_IN_WITH_PLATFORM = `
  mutation SignInWithPlatform($platformId: String!, $accessToken: String!, $allowSignUp: Boolean) {
    signInWithPlatform(platformId: $platformId, accessToken: $accessToken, allowSignUp: $allowSignUp) {
      ${USER_FIELDS}
    }
  }
`;

const VERIFY = `
  mutation Verify($token: String!) {
    verify(token: $token) {
      ${USER_FIELDS}
    }
  }
`;

const CREATE_USER = `
  mutation CreateUser($input: CreateUserInput!) {
    createUser(input: $input) {
      ${USER_FIELDS}
    }
  }
`;

const UPDATE_USER = `
  mutation UpdateUser($input: UpdateUserInput!) {
    updateUser(input: $input) {
      ${USER_FIELDS}
    }
  }
`;

const ME = `
  query Me {
    me {
      ${USER_FIELDS}
    }
  }
`;

export class AuthOperations {
  private readonly client: GraphQLClient;

  constructor(client: GraphQLClient) {
    this.client = client;
  }

  async signInWithPlatform(input: SignInWithPlatformInput): Promise<AuthResult> {
    const response = await this.client.mutate<{ signInWithPlatform: UserProfile }>(
      SIGN_IN_WITH_PLATFORM,
      {
        platformId: input.platformId,
        accessToken: input.accessToken,
        allowSignUp: input.allowSignUp,
      },
    );
    return { user: response.signInWithPlatform };
  }

  async verify(token: string): Promise<AuthResult> {
    const response = await this.client.mutate<{ verify: UserProfile }>(
      VERIFY,
      { token },
    );
    return { user: response.verify };
  }

  async me(): Promise<UserProfile> {
    const response = await this.client.query<{ me: UserProfile }>(ME);
    return response.me;
  }

  async updateProfile(input: UpdateProfileInput): Promise<UserProfile> {
    const response = await this.client.mutate<{ updateUser: UserProfile }>(
      UPDATE_USER,
      { input },
    );
    return response.updateUser;
  }

  async createUser(input: CreateUserInput): Promise<UserProfile> {
    const response = await this.client.mutate<{ createUser: UserProfile }>(
      CREATE_USER,
      { input },
    );
    return response.createUser;
  }
}
