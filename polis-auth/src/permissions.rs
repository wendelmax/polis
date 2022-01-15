use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PermissionManager {
    permissions: HashMap<String, Permission>,
    roles: HashMap<String, Role>,
    user_roles: HashMap<Uuid, Vec<String>>,
    user_permissions: HashMap<Uuid, Vec<String>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            permissions: HashMap::new(),
            roles: HashMap::new(),
            user_roles: HashMap::new(),
            user_permissions: HashMap::new(),
        };

        // Inicializar permissões e roles padrão
        manager.initialize_default_permissions();
        manager
    }

    fn initialize_default_permissions(&mut self) {
        // Permissões básicas
        let permissions = vec![
            Permission {
                id: "containers:read".to_string(),
                name: "Listar Containers".to_string(),
                description: "Visualizar lista de containers".to_string(),
                resource: "containers".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "containers:create".to_string(),
                name: "Criar Container".to_string(),
                description: "Criar novos containers".to_string(),
                resource: "containers".to_string(),
                action: "create".to_string(),
            },
            Permission {
                id: "containers:update".to_string(),
                name: "Atualizar Container".to_string(),
                description: "Atualizar containers existentes".to_string(),
                resource: "containers".to_string(),
                action: "update".to_string(),
            },
            Permission {
                id: "containers:delete".to_string(),
                name: "Deletar Container".to_string(),
                description: "Deletar containers".to_string(),
                resource: "containers".to_string(),
                action: "delete".to_string(),
            },
            Permission {
                id: "images:read".to_string(),
                name: "Listar Imagens".to_string(),
                description: "Visualizar lista de imagens".to_string(),
                resource: "images".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "images:create".to_string(),
                name: "Criar Imagem".to_string(),
                description: "Criar novas imagens".to_string(),
                resource: "images".to_string(),
                action: "create".to_string(),
            },
            Permission {
                id: "images:delete".to_string(),
                name: "Deletar Imagem".to_string(),
                description: "Deletar imagens".to_string(),
                resource: "images".to_string(),
                action: "delete".to_string(),
            },
            Permission {
                id: "networks:read".to_string(),
                name: "Listar Redes".to_string(),
                description: "Visualizar lista de redes".to_string(),
                resource: "networks".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "networks:create".to_string(),
                name: "Criar Rede".to_string(),
                description: "Criar novas redes".to_string(),
                resource: "networks".to_string(),
                action: "create".to_string(),
            },
            Permission {
                id: "networks:delete".to_string(),
                name: "Deletar Rede".to_string(),
                description: "Deletar redes".to_string(),
                resource: "networks".to_string(),
                action: "delete".to_string(),
            },
            Permission {
                id: "volumes:read".to_string(),
                name: "Listar Volumes".to_string(),
                description: "Visualizar lista de volumes".to_string(),
                resource: "volumes".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "volumes:create".to_string(),
                name: "Criar Volume".to_string(),
                description: "Criar novos volumes".to_string(),
                resource: "volumes".to_string(),
                action: "create".to_string(),
            },
            Permission {
                id: "volumes:delete".to_string(),
                name: "Deletar Volume".to_string(),
                description: "Deletar volumes".to_string(),
                resource: "volumes".to_string(),
                action: "delete".to_string(),
            },
            Permission {
                id: "system:read".to_string(),
                name: "Visualizar Sistema".to_string(),
                description: "Visualizar informações do sistema".to_string(),
                resource: "system".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "system:admin".to_string(),
                name: "Administrar Sistema".to_string(),
                description: "Administrar configurações do sistema".to_string(),
                resource: "system".to_string(),
                action: "admin".to_string(),
            },
        ];

        for permission in permissions {
            self.permissions.insert(permission.id.clone(), permission);
        }

        // Roles padrão
        let roles = vec![
            Role {
                id: "admin".to_string(),
                name: "Administrador".to_string(),
                description: "Acesso total ao sistema".to_string(),
                permissions: vec![
                    "containers:read".to_string(),
                    "containers:create".to_string(),
                    "containers:update".to_string(),
                    "containers:delete".to_string(),
                    "images:read".to_string(),
                    "images:create".to_string(),
                    "images:delete".to_string(),
                    "networks:read".to_string(),
                    "networks:create".to_string(),
                    "networks:delete".to_string(),
                    "volumes:read".to_string(),
                    "volumes:create".to_string(),
                    "volumes:delete".to_string(),
                    "system:read".to_string(),
                    "system:admin".to_string(),
                ],
            },
            Role {
                id: "user".to_string(),
                name: "Usuário".to_string(),
                description: "Acesso básico ao sistema".to_string(),
                permissions: vec![
                    "containers:read".to_string(),
                    "containers:create".to_string(),
                    "containers:update".to_string(),
                    "images:read".to_string(),
                    "images:create".to_string(),
                    "networks:read".to_string(),
                    "volumes:read".to_string(),
                    "volumes:create".to_string(),
                    "system:read".to_string(),
                ],
            },
            Role {
                id: "viewer".to_string(),
                name: "Visualizador".to_string(),
                description: "Apenas visualização".to_string(),
                permissions: vec![
                    "containers:read".to_string(),
                    "images:read".to_string(),
                    "networks:read".to_string(),
                    "volumes:read".to_string(),
                    "system:read".to_string(),
                ],
            },
        ];

        for role in roles {
            self.roles.insert(role.id.clone(), role);
        }

        // Atribuir role de admin ao usuário admin padrão
        // Isso será feito quando o usuário for criado
    }

    pub async fn assign_role_to_user(&mut self, user_id: Uuid, role_id: &str) -> Result<()> {
        if !self.roles.contains_key(role_id) {
            return Err(PolisError::Auth("Role não encontrada".to_string()));
        }

        let role = self.roles.get(role_id).unwrap();
        let user_permissions = self
            .user_permissions
            .entry(user_id)
            .or_insert_with(Vec::new);

        // Adicionar permissões da role
        for permission_id in &role.permissions {
            if !user_permissions.contains(permission_id) {
                user_permissions.push(permission_id.clone());
            }
        }

        // Adicionar role ao usuário
        let user_roles = self.user_roles.entry(user_id).or_insert_with(Vec::new);
        if !user_roles.contains(&role_id.to_string()) {
            user_roles.push(role_id.to_string());
        }

        Ok(())
    }

    pub async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<String>> {
        Ok(self
            .user_permissions
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn check_permission(&self, user_id: &Uuid, permission: &str) -> Result<bool> {
        let user_permissions = self.get_user_permissions(user_id).await?;
        Ok(user_permissions.contains(&permission.to_string()))
    }

    pub async fn add_permission_to_user(
        &mut self,
        user_id: Uuid,
        permission_id: &str,
    ) -> Result<()> {
        if !self.permissions.contains_key(permission_id) {
            return Err(PolisError::Auth("Permissão não encontrada".to_string()));
        }

        let user_permissions = self
            .user_permissions
            .entry(user_id)
            .or_insert_with(Vec::new);
        if !user_permissions.contains(&permission_id.to_string()) {
            user_permissions.push(permission_id.to_string());
        }

        Ok(())
    }

    pub async fn remove_permission_from_user(
        &mut self,
        user_id: Uuid,
        permission_id: &str,
    ) -> Result<()> {
        if let Some(user_permissions) = self.user_permissions.get_mut(&user_id) {
            user_permissions.retain(|p| p != permission_id);
        }

        Ok(())
    }

    pub async fn list_permissions(&self) -> Result<Vec<Permission>> {
        Ok(self.permissions.values().cloned().collect())
    }

    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        Ok(self.roles.values().cloned().collect())
    }

    pub async fn create_permission(
        &mut self,
        id: String,
        name: String,
        description: String,
        resource: String,
        action: String,
    ) -> Result<Permission> {
        if self.permissions.contains_key(&id) {
            return Err(PolisError::Auth("Permissão já existe".to_string()));
        }

        let permission = Permission {
            id: id.clone(),
            name,
            description,
            resource,
            action,
        };

        self.permissions.insert(id, permission.clone());
        Ok(permission)
    }

    pub async fn create_role(
        &mut self,
        id: String,
        name: String,
        description: String,
        permissions: Vec<String>,
    ) -> Result<Role> {
        if self.roles.contains_key(&id) {
            return Err(PolisError::Auth("Role já existe".to_string()));
        }

        // Verificar se todas as permissões existem
        for permission_id in &permissions {
            if !self.permissions.contains_key(permission_id) {
                return Err(PolisError::Auth(format!(
                    "Permissão não encontrada: {}",
                    permission_id
                )));
            }
        }

        let role = Role {
            id: id.clone(),
            name,
            description,
            permissions,
        };

        self.roles.insert(id, role.clone());
        Ok(role)
    }
}
